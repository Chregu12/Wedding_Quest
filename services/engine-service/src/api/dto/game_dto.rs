use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request types ---

#[derive(Debug, Deserialize)]
pub struct SubmitAnswerRequest {
    pub player_id: Uuid,
    pub player_name: String,
    pub answer: String,
    #[serde(default)]
    pub couple: bool,
}

#[derive(Debug, Deserialize)]
pub struct CoupleAnswerRequest {
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct CoupleIndividualAnswerRequest {
    pub person: String, // "a" or "b"
    pub answer: String, // "ich" or "du"
}

#[derive(Debug, Serialize)]
pub struct CoupleIndividualAnswerResponse {
    pub both_answered: bool,
    pub agree: Option<bool>,
    pub final_answer: Option<String>,
}

// --- Response types ---

#[derive(Debug, Serialize)]
pub struct StartGameResponse {
    pub round_id: Uuid,
    pub question_text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub round_number: i32,
    pub total_questions: i32,
}

#[derive(Debug, Serialize)]
pub struct GameStateResponse {
    pub status: String,
    pub current_round_id: Option<Uuid>,
    pub current_round_number: i32,
    pub total_questions: i32,
    pub question_type: Option<String>,
    pub question_text: Option<String>,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub started_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitAnswerResponse {
    pub accepted: bool,
    pub is_correct: bool,
}

#[derive(Debug, Serialize)]
pub struct CloseRoundResponse {
    pub correct_answer: String,
    pub has_ich_oder_du: bool,
    pub ich_oder_du_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NextQuestionResponse {
    pub round_id: Uuid,
    pub question_text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub round_number: i32,
    pub total_questions: i32,
}

#[derive(Debug, Serialize)]
pub struct RoundAnswerResponse {
    pub player_id: Uuid,
    pub player_name: String,
    pub answer: String,
    pub is_correct: bool,
    pub time_taken_seconds: f64,
    pub question_type: String,
}
