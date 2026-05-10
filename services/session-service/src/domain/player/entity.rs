use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::events::PlayerDomainEvent;
use super::value_objects::PlayerName;

pub struct Player {
    pub id: Uuid,
    pub session_id: Uuid,
    pub display_name: PlayerName,
    pub avatar: Option<String>,
    pub total_score: i64,
    pub is_connected: bool,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub(crate) pending_events: Vec<PlayerDomainEvent>,
}

impl Player {
    pub fn join(session_id: Uuid, session_code: String, name: PlayerName) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let event = PlayerDomainEvent::PlayerJoined {
            player_id: id,
            session_id,
            session_code,
            display_name: name.value().to_string(),
            occurred_at: now,
        };

        Self {
            id,
            session_id,
            display_name: name,
            avatar: None,
            total_score: 0,
            is_connected: true,
            joined_at: now,
            updated_at: now,
            pending_events: vec![event],
        }
    }

    pub fn take_pending_events(&mut self) -> Vec<PlayerDomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
