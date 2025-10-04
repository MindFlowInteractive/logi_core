use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: String,
    pub username: String,
    pub score: u32,
    pub category: String,
    pub difficulty: String,
    pub region: String,
    pub timestamp: i64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardFilter {
    pub category: Option<String>,
    pub difficulty: Option<String>,
    pub region: Option<String>,
    pub period: Option<String>, // e.g., "daily", "weekly", "monthly", "all-time"
}
