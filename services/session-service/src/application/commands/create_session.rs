use crate::domain::game_session::entity::GameSession;
use crate::domain::game_session::repository::GameSessionRepository;
use crate::errors::AppError;
use crate::infrastructure::messaging::EventPublisher;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionCommand {
    pub person_a_name: String,
    pub person_b_name: String,
    pub host_name: String,
}

pub struct CreateSessionResult {
    pub session_id: Uuid,
    pub code: String,
}

pub struct CreateSessionHandler<R, P> {
    session_repo: R,
    event_publisher: P,
}

impl<R, P> CreateSessionHandler<R, P>
where
    R: GameSessionRepository,
    P: EventPublisher,
{
    pub fn new(session_repo: R, event_publisher: P) -> Self {
        Self { session_repo, event_publisher }
    }

    pub async fn handle(&self, cmd: CreateSessionCommand) -> Result<CreateSessionResult, AppError> {
        let mut session = GameSession::create(
            cmd.person_a_name,
            cmd.person_b_name,
            cmd.host_name,
        );

        let result = CreateSessionResult {
            session_id: session.id,
            code: session.code.value().to_string(),
        };

        self.session_repo.save(&session).await?;

        let events = session.take_pending_events();
        for event in events {
            self.event_publisher.publish_session_event(event).await?;
        }

        Ok(result)
    }
}
