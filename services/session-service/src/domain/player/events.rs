use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerDomainEvent {
    PlayerJoined {
        player_id: Uuid,
        session_id: Uuid,
        session_code: String,
        display_name: String,
        occurred_at: DateTime<Utc>,
    },
    PlayerDisconnected {
        player_id: Uuid,
        session_id: Uuid,
        session_code: String,
        occurred_at: DateTime<Utc>,
    },
}

impl PlayerDomainEvent {
    pub fn session_code(&self) -> &str {
        match self {
            PlayerDomainEvent::PlayerJoined { session_code, .. } => session_code,
            PlayerDomainEvent::PlayerDisconnected { session_code, .. } => session_code,
        }
    }

    pub fn channel(&self) -> String {
        format!("wedding_quest:session:{}", self.session_code())
    }
}
