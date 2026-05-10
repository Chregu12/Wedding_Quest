use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Status of an individual game round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundStatus {
    /// Accepting player answers.
    Active,
    /// Admin closed; waiting for couple-answer if ich_oder_du is present.
    IchOderDu,
    /// Round fully scored; ready for next-question.
    Scored,
}

impl RoundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::IchOderDu => "ich_oder_du",
            Self::Scored => "scored",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(Self::Active),
            "ich_oder_du" => Ok(Self::IchOderDu),
            "scored" => Ok(Self::Scored),
            other => Err(format!("Unknown round status: {other}")),
        }
    }
}

/// Overall game status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStatus {
    Waiting,
    Question,
    IchOderDu,
    Finished,
}

impl GameStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Question => "question",
            Self::IchOderDu => "ich_oder_du",
            Self::Finished => "finished",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "waiting" => Ok(Self::Waiting),
            "question" => Ok(Self::Question),
            "ich_oder_du" => Ok(Self::IchOderDu),
            "finished" => Ok(Self::Finished),
            other => Err(format!("Unknown game status: {other}")),
        }
    }
}

/// Domain entity for a single game round.
#[derive(Debug, Clone)]
pub struct GameRound {
    pub id: Uuid,
    pub session_code: String,
    pub question_id: Uuid,
    pub question_type: String,
    pub question_text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    /// Paired ich-oder-du question id (if any).
    pub ich_oder_du_id: Option<Uuid>,
    /// Paired ich-oder-du question text (if any).
    pub ich_oder_du_text: Option<String>,
    /// Correct answer for the ich-oder-du phase (if any).
    pub ich_oder_du_correct: Option<String>,
    /// Answer submitted by the couple during ich-oder-du phase.
    pub couple_answer: Option<String>,
    pub status: RoundStatus,
    pub round_number: i32,
    pub started_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Domain entity for a player's answer in a round.
#[derive(Debug, Clone)]
pub struct PlayerAnswer {
    pub id: Uuid,
    pub round_id: Uuid,
    pub player_id: Uuid,
    pub player_name: String,
    pub answer: String,
    pub is_correct: bool,
    pub answered_at: DateTime<Utc>,
    pub time_taken_seconds: f64,
}

/// Domain entity for the overall game state.
#[derive(Debug, Clone)]
pub struct GameState {
    pub session_code: String,
    pub status: GameStatus,
    pub current_round_id: Option<Uuid>,
    pub current_round_number: i32,
    pub total_questions: i32,
    pub updated_at: DateTime<Utc>,
}
