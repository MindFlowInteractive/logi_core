
mod leaderboard;

use actix_web::{web, App, HttpServer};
use leaderboard::handlers::get_leaderboard;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/leaderboard", web::get().to(get_leaderboard))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
