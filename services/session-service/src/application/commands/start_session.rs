use crate::domain::game_session::repository::GameSessionRepository;
use crate::errors::AppError;
use crate::infrastructure::messaging::EventPublisher;
use uuid::Uuid;

pub struct StartSessionCommand {
    pub session_id: Uuid,
}

pub struct StartSessionHandler<R, P> {
    session_repo: R,
    event_publisher: P,
}

impl<R, P> StartSessionHandler<R, P>
where
    R: GameSessionRepository,
    P: EventPublisher,
{
    pub fn new(session_repo: R, event_publisher: P) -> Self {
        Self { session_repo, event_publisher }
    }

    pub async fn handle(&self, cmd: StartSessionCommand) -> Result<(), AppError> {
        let mut session = self
            .session_repo
            .find_by_id(cmd.session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        session.start().map_err(AppError::Domain)?;

        self.session_repo.update(&session).await?;

        let events = session.take_pending_events();
        for event in events {
            self.event_publisher.publish_session_event(event).await?;
        }

        Ok(())
    }
}
