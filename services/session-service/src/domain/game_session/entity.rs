use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::events::SessionDomainEvent;
use super::value_objects::{GameCode, SessionStatus};
use crate::errors::DomainError;

pub struct GameSession {
    pub id: Uuid,
    pub game_id: Option<Uuid>,
    pub code: GameCode,
    pub status: SessionStatus,
    pub host_name: String,
    pub person_a_name: String,
    pub person_b_name: String,
    pub current_round: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub(crate) pending_events: Vec<SessionDomainEvent>,
}

impl GameSession {
    pub fn create(
        person_a_name: String,
        person_b_name: String,
        host_name: String,
    ) -> Self {
        let id = Uuid::new_v4();
        let code = GameCode::generate();
        let now = Utc::now();

        let event = SessionDomainEvent::SessionCreated {
            session_id: id,
            code: code.value().to_string(),
            person_a_name: person_a_name.clone(),
            person_b_name: person_b_name.clone(),
            occurred_at: now,
        };

        Self {
            id,
            game_id: None,
            code,
            status: SessionStatus::Lobby,
            host_name,
            person_a_name,
            person_b_name,
            current_round: None,
            started_at: None,
            ended_at: None,
            created_at: now,
            updated_at: now,
            pending_events: vec![event],
        }
    }

    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.status != SessionStatus::Lobby {
            return Err(DomainError::SessionNotInLobby);
        }
        let now = Utc::now();
        self.status = SessionStatus::Active;
        self.started_at = Some(now);
        self.updated_at = now;
        self.pending_events.push(SessionDomainEvent::SessionStarted {
            session_id: self.id,
            code: self.code.value().to_string(),
            occurred_at: now,
        });
        Ok(())
    }

    pub fn take_pending_events(&mut self) -> Vec<SessionDomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
