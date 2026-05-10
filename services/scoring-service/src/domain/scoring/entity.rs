use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Aggregate score for a player within a session.
#[derive(Debug, Clone)]
pub struct PlayerScore {
    pub id: Uuid,
    pub session_code: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub total_score: i32,
    pub rounds_played: i32,
    pub last_round_score: i32,
    pub updated_at: DateTime<Utc>,
}

impl PlayerScore {
    pub fn new(session_code: String, player_id: Uuid, player_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_code,
            player_id,
            player_name,
            total_score: 0,
            rounds_played: 0,
            last_round_score: 0,
            updated_at: Utc::now(),
        }
    }

    pub fn apply_round_score(&mut self, round_points: i32) {
        self.total_score += round_points;
        self.rounds_played += 1;
        self.last_round_score = round_points;
        self.updated_at = Utc::now();
    }
}

/// Score earned by a player in a single round.
#[derive(Debug, Clone)]
pub struct RoundScore {
    pub id: Uuid,
    pub round_id: Uuid,
    pub session_code: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub base_points: i32,
    pub time_multiplier: f64,
    pub final_points: i32,
    pub is_correct: bool,
}

impl RoundScore {
    pub fn new(
        round_id: Uuid,
        session_code: String,
        player_id: Uuid,
        player_name: String,
        base_points: i32,
        time_multiplier: f64,
        final_points: i32,
        is_correct: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            round_id,
            session_code,
            player_id,
            player_name,
            base_points,
            time_multiplier,
            final_points,
            is_correct,
        }
    }
}
