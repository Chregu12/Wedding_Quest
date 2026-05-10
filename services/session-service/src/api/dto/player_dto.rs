use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct JoinSessionRequest {
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinSessionResponse {
    pub player_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PlayerResponse {
    pub id: Uuid,
    pub display_name: String,
    pub avatar: Option<String>,
    pub total_score: i64,
    pub is_connected: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LobbyResponse {
    pub session_code: String,
    pub person_a_name: String,
    pub person_b_name: String,
    pub players: Vec<PlayerResponse>,
}
