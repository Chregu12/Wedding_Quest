use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AddGuestQuizRequest {
    pub text: String,
    pub option_a: String,
    pub option_b: String,
    pub option_c: String,
    pub option_d: String,
    pub correct_answer: String, // "A"|"B"|"C"|"D"
    pub order_index: Option<i32>,
    pub points: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddIchOderDuRequest {
    pub text: String,
    pub correct_answer: String, // "ich"|"du"
    pub order_index: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct QuestionResponse {
    pub id: Uuid,
    pub question_type: String,
    pub text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub order_index: i32,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScoreConfigRequest {
    pub tier1_max_seconds: Option<i32>,
    pub tier2_max_seconds: Option<i32>,
    pub tier1_multiplier: Option<f64>,
    pub tier2_multiplier: Option<f64>,
    pub tier3_multiplier: Option<f64>,
    pub perfect_match_multiplier: Option<f64>,
    pub catchup_multiplier: Option<f64>,
    pub catchup_threshold_percent: Option<i32>,
    pub base_points: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ScoreConfigResponse {
    pub session_id: Uuid,
    pub tier1_max_seconds: i32,
    pub tier2_max_seconds: i32,
    pub tier1_multiplier: f64,
    pub tier2_multiplier: f64,
    pub tier3_multiplier: f64,
    pub perfect_match_multiplier: f64,
    pub catchup_multiplier: f64,
    pub catchup_threshold_percent: i32,
    pub base_points: i32,
}
