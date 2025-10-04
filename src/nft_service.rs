use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use reqwest::Client;
use anyhow::{anyhow, Result};
use chrono::{Utc, DateTime};
use jsonschema::{Draft, JSONSchema};
use std::env;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Debug, Deserialize)]
struct GenerateRequest {
    // unique identifier for the user or submission (optional)
    owner_id: Option<String>,

    // Puzzle stats
    total_puzzles: u32,
    puzzles_solved: u32,
    best_time_seconds: Option<u32>,
    average_time_seconds: Option<u32>,

    // discipline e.g., "logic", "combinatorics", "graph", "algebra"
    discipline: String,

    // difficulty label: "Easy", "Medium", "Hard", "Expert"
    difficulty: String,

    // completion timestamp (optional). If absent, server uses now.
    completion_timestamp: Option<DateTime<Utc>>,

    // extra metadata fields (optional)
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct Attribute {
    trait_type: String,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct NFTMetadata {
    name: String,
    description: String,
    image: String, // should be IPFS URI (ipfs://<cid> or https gateway)
    external_url: Option<String>,
    attributes: Vec<Attribute>,

    // marketplace-agnostic custom fields
    #[serde(skip_serializing_if = "Option::is_none")]
    puzzle_stats: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completion_timestamp: Option<DateTime<Utc>>,
    difficulty: String,
    discipline: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    tracing_subscriber::fmt::init();

    // require WEB3_STORAGE_TOKEN in env
    if env::var("WEB3_STORAGE_TOKEN").is_err() {
        error!("WEB3_STORAGE_TOKEN env var is missing. Set it before running.");
        // still continue so devs can run validation-only flows if they want
    }

    let app = Router::new().route("/generate", post(generate_handler));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse::<u16>().unwrap_or(8080);
    info!("Listening on 0.0.0.0:{} ...", port);

    axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn generate_handler(Json(req): Json<GenerateRequest>) -> impl IntoResponse {
    match generate_and_store(req).await {
        Ok(metadata_cid) => {
            let metadata_uri = format!("ipfs://{}", metadata_cid);
            (StatusCode::CREATED, Json(json!({"metadata_uri": metadata_uri})))
        }
        Err(e) => {
            error!("Error generating metadata: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{:?}", e)})))
        }
    }
}

async fn generate_and_store(req: GenerateRequest) -> Result<String> {
    let client = Client::new();

    // 1. Compose an SVG image for the discipline + stats
    let svg = build_svg_for_discipline(
        &req.discipline,
        req.total_puzzles,
        req.puzzles_solved,
        req.best_time_seconds,
        &req.difficulty,
    );

    // 2. Optionally minify or convert to PNG in the future. For now we'll store the SVG as bytes.
    let image_bytes = svg.into_bytes();

    // 3. Upload image bytes to IPFS (web3.storage)
    let image_cid = ipfs_upload_bytes(&client, &image_bytes, "art.svg").await?;
    info!("Uploaded image to IPFS: {}", image_cid);

    // 4. Compose metadata JSON (marketplace friendly)
    let completion_ts = req.completion_timestamp.unwrap_or_else(|| Utc::now());
    let name = format!("Achievement NFT — {} Mastery", capitalize_first(&req.discipline));
    let description = format!(
        "Achievement awarded for mastering {} puzzles in the {} discipline. Difficulty: {}. Generated automatically.",
        req.puzzles_solved, req.discipline, req.difficulty
    );

    let image_uri = format!("ipfs://{}", image_cid); // recommended for NFT metadata

    let mut attributes = vec![
        Attribute { trait_type: "Discipline".into(), value: json!(req.discipline.clone()), display_type: None },
        Attribute { trait_type: "Difficulty".into(), value: json!(req.difficulty.clone()), display_type: None },
        Attribute { trait_type: "Puzzles Solved".into(), value: json!(req.puzzles_solved), display_type: Some("number".into()) },
        Attribute { trait_type: "Total Puzzles".into(), value: json!(req.total_puzzles), display_type: Some("number".into()) },
    ];

    if let Some(best) = req.best_time_seconds {
        attributes.push(Attribute { trait_type: "Best Time (s)".into(), value: json!(best), display_type: Some("number".into()) });
    }
    if let Some(avg) = req.average_time_seconds {
        attributes.push(Attribute { trait_type: "Average Time (s)".into(), value: json!(avg), display_type: Some("number".into()) });
    }

    if let Some(notes) = &req.notes {
        attributes.push(Attribute { trait_type: "Notes".into(), value: json!(notes), display_type: None });
    }

    // extra puzzle_stats object for deeper indexers / your dApp
    let puzzle_stats = json!({
        "total_puzzles": req.total_puzzles,
        "puzzles_solved": req.puzzles_solved,
        "best_time_seconds": req.best_time_seconds,
        "average_time_seconds": req.average_time_seconds
    });

    let metadata = NFTMetadata {
        name,
        description,
        image: image_uri,
        external_url: None,
        attributes,
        puzzle_stats: Some(puzzle_stats),
        completion_timestamp: Some(completion_ts),
        difficulty: req.difficulty.clone(),
        discipline: req.discipline.clone(),
        extra: None,
    };

    // 5. Validate against JSON schema
    validate_metadata_schema(&metadata)?;

    // 6. Serialize metadata and upload to IPFS
    let metadata_json = serde_json::to_vec_pretty(&metadata)?;
    let metadata_cid = ipfs_upload_bytes(&client, &metadata_json, "metadata.json").await?;
    info!("Uploaded metadata to IPFS: {}", metadata_cid);

    Ok(metadata_cid)
}

/// Simple SVG generator — returns a string containing SVG.
/// You can replace this with a call to an image generation service or a more complex renderer.
/// Aim: generate discipline-specific badges using shapes + text.
fn build_svg_for_discipline(
    discipline: &str,
    total: u32,
    solved: u32,
    best_time: Option<u32>,
    difficulty: &str,
) -> String {
    // Choose palette by discipline
    let (bg, accent) = match discipline.to_lowercase().as_str() {
        "logic" => ("#0f172a", "#60a5fa"),       // slate + blue
        "combinatorics" => ("#042f2e", "#34d399"), // teal + green
        "graph" => ("#1f2937", "#f472b6"),       // gray + pink
        "algebra" => ("#0f172a", "#f59e0b"),     // slate + amber
        _ => ("#111827", "#60a5fa"),
    };

    let percent = if total == 0 { 0 } else { (solved * 100 / total) };

    // Use inline SVG with simple bar and texts
    let best_text = best_time.map(|s| format!("Best: {}s", s)).unwrap_or_else(|| "".into());

    format!(
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='1200' height='1200' viewBox='0 0 1200 1200'>
  <defs>
    <linearGradient id='g' x1='0' x2='1'>
      <stop offset='0' stop-color='{accent}' stop-opacity='0.9'/>
      <stop offset='1' stop-color='#000000' stop-opacity='0.5'/>
    </linearGradient>
  </defs>
  <rect width='100%' height='100%' fill='{bg}'/>
  <g transform='translate(100,120)'>
    <rect width='1000' height='1000' rx='48' fill='url(#g)' opacity='0.14'/>
    <text x='60' y='140' font-family='Arial' font-size='48' fill='white'>Achievement: {discipline}</text>
    <text x='60' y='200' font-family='Arial' font-size='34' fill='white'>Difficulty: {difficulty}</text>
    <g transform='translate(60,260)'>
      <rect width='1080' height='80' rx='40' fill='#0b1220' />
      <rect width='{bar_width}' height='80' rx='40' fill='{accent}' />
      <text x='540' y='52' text-anchor='middle' font-family='Arial' font-size='32' fill='white'>{percent}% Complete</text>
    </g>
    <g transform='translate(60,380)'>
      <text x='0' y='52' font-family='Arial' font-size='28' fill='white'>Solved: {solved} / {total}</text>
      <text x='0' y='92' font-family='Arial' font-size='24' fill='white'>{best_text}</text>
    </g>
    <g transform='translate(60,500)'>
      <text x='0' y='40' font-family='Arial' font-size='20' fill='white'>Generated: {now}</text>
      <text x='0' y='72' font-family='Arial' font-size='20' fill='white'>Token ID: {uuid}</text>
    </g>
  </g>
</svg>"#,
        accent = accent,
        bg = bg,
        discipline = capitalize_first(discipline),
        difficulty = difficulty,
        solved = solved,
        total = total,
        best_text = best_text,
        percent = percent,
        bar_width = (1080 * percent / 100), // integer math OK
        now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        uuid = Uuid::new_v4(),
    )
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

/// Upload arbitrary bytes to IPFS via web3.storage (multipart file).
/// Returns CID string on success.
/// Expects WEB3_STORAGE_TOKEN env var to be set.
async fn ipfs_upload_bytes(client: &Client, bytes: &[u8], filename: &str) -> Result<String> {
    let token = env::var("WEB3_STORAGE_TOKEN").map_err(|_| anyhow!("WEB3_STORAGE_TOKEN not set"))?;
    // web3.storage upload endpoint:
    // POST https://api.web3.storage/upload
    let url = "https://api.web3.storage/upload";

    let part = reqwest::multipart::Part::bytes(bytes.to_vec()).file_name(filename.to_string());

    let form = reqwest::multipart::Form::new().part("file", part);

    let resp = client
        .post(url)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?;

    let body: serde_json::Value = resp.json().await?;
    // web3.storage returns "cid" or "cid" inside "value" depending on API version; handle both
    if let Some(cid) = body.get("cid").and_then(|v| v.as_str()) {
        Ok(cid.to_string())
    } else if let Some(value) = body.get("value").and_then(|v| v.get("cid")).and_then(|v| v.as_str()) {
        Ok(value.to_string())
    } else {
        Err(anyhow!("Unexpected IPFS upload response: {}", body))
    }
}

fn validate_metadata_schema(metadata: &NFTMetadata) -> Result<()> {
    // Build a JSON schema that is compatible with common marketplaces.
    // The schema below enforces: name (string), description (string), image (string), attributes (array of trait objects)
    // and checks for our custom fields. You can expand it with more detailed rules if needed.
    let schema_json = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Achievement NFT Metadata Schema",
        "type": "object",
        "required": ["name", "description", "image", "attributes", "difficulty", "discipline"],
        "properties": {
            "name": { "type": "string" },
            "description": { "type": "string" },
            "image": { "type": "string" },
            "external_url": { "type": ["string", "null"] },
            "attributes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["trait_type", "value"],
                    "properties": {
                        "trait_type": { "type": "string" },
                        "value": {},
                        "display_type": { "type": "string" }
                    }
                }
            },
            "puzzle_stats": {
                "type": "object",
                "properties": {
                    "total_puzzles": { "type": "integer" },
                    "puzzles_solved": { "type": "integer" },
                    "best_time_seconds": { "type": ["integer", "null"] },
                    "average_time_seconds": { "type": ["integer", "null"] }
                }
            },
            "completion_timestamp": { "type": "string", "format": "date-time" },
            "difficulty": { "type": "string" },
            "discipline": { "type": "string" }
        },
        "additionalProperties": true
    });

    // compile and validate
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| anyhow!("Schema compile error: {}", e))?;

    let instance = serde_json::to_value(metadata)?;
    let result = compiled.validate(&instance);

    if let Err(errors) = result {
        // collect first few errors
        let msgs: Vec<String> = errors.take(10).map(|e| e.to_string()).collect();
        return Err(anyhow!("Metadata validation failed: {:?}", msgs));
    }

    Ok(())
}
