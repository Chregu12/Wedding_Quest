use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionDomainEvent {
    SessionCreated {
        session_id: Uuid,
        code: String,
        person_a_name: String,
        person_b_name: String,
        occurred_at: DateTime<Utc>,
    },
    SessionStarted {
        session_id: Uuid,
        code: String,
        occurred_at: DateTime<Utc>,
    },
    SessionEnded {
        session_id: Uuid,
        occurred_at: DateTime<Utc>,
    },
}

impl SessionDomainEvent {
    pub fn session_id(&self) -> Uuid {
        match self {
            SessionDomainEvent::SessionCreated { session_id, .. } => *session_id,
            SessionDomainEvent::SessionStarted { session_id, .. } => *session_id,
            SessionDomainEvent::SessionEnded { session_id, .. } => *session_id,
        }
    }

    pub fn channel(&self) -> String {
        format!("wedding_quest:session:{}", self.session_id())
    }
}
