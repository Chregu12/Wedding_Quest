use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PlayerScoreResponse {
    pub player_id: Uuid,
    pub player_name: String,
    pub total_score: i32,
    pub rounds_played: i32,
    pub last_round_score: i32,
    pub rank: usize,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub session_code: String,
    pub scores: Vec<PlayerScoreResponse>,
}
