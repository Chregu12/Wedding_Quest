use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub person_a_name: String,
    pub person_b_name: String,
    pub host_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub code: String,
    pub status: String,
    pub person_a_name: String,
    pub person_b_name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
