use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameEvent {
    QuestionStarted {
        round_id: Uuid,
        question_id: Uuid,
        question_type: String,
        question_text: String,
        option_a: Option<String>,
        option_b: Option<String>,
        option_c: Option<String>,
        option_d: Option<String>,
        correct_answer: String,
        started_at: DateTime<Utc>,
        round_number: i32,
    },
    RoundClosed {
        round_id: Uuid,
        correct_answer: String,
        closed_at: DateTime<Utc>,
    },
    IchOderDuStarted {
        round_id: Uuid,
        ich_oder_du_text: String,
    },
    CoupleAnswered {
        round_id: Uuid,
        couple_answer: String,
    },
    GameEnded {
        session_code: String,
    },
}

impl GameEvent {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
