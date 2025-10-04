use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};

use crate::leaderboard::model::{LeaderboardEntry, LeaderboardFilter};

// Dummy data for demonstration
fn get_dummy_leaderboard() -> Vec<LeaderboardEntry> {
    vec![
        LeaderboardEntry {
            user_id: "1".to_string(),
            username: "Alice".to_string(),
            score: 1200,
            category: "Logic".to_string(),
            difficulty: "Easy".to_string(),
            region: "NA".to_string(),
            timestamp: Utc::now().timestamp(),
        },
        LeaderboardEntry {
            user_id: "2".to_string(),
            username: "Bob".to_string(),
            score: 1100,
            category: "Math".to_string(),
            difficulty: "Medium".to_string(),
            region: "EU".to_string(),
            timestamp: Utc::now().timestamp(),
        },
    ]
}

pub async fn get_leaderboard(
    filter: web::Query<LeaderboardFilter>,
) -> impl Responder {
    let mut entries = get_dummy_leaderboard();

    if let Some(ref cat) = filter.category {
        entries = entries.into_iter().filter(|e| &e.category == cat).collect();
    }
    if let Some(ref diff) = filter.difficulty {
        entries = entries.into_iter().filter(|e| &e.difficulty == diff).collect();
    }
    if let Some(ref reg) = filter.region {
        entries = entries.into_iter().filter(|e| &e.region == reg).collect();
    }
    if let Some(ref period) = filter.period {
        let now = Utc::now().timestamp();
        let cutoff = match period.as_str() {
            "daily" => now - Duration::days(1).num_seconds(),
            "weekly" => now - Duration::weeks(1).num_seconds(),
            "monthly" => now - Duration::days(30).num_seconds(),
            _ => 0,
        };
        entries = entries.into_iter().filter(|e| e.timestamp >= cutoff).collect();
    }

    HttpResponse::Ok().json(entries)
}
