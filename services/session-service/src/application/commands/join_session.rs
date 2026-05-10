use crate::domain::game_session::repository::GameSessionRepository;
use crate::domain::game_session::value_objects::{GameCode, SessionStatus};
use crate::domain::player::entity::Player;
use crate::domain::player::repository::PlayerRepository;
use crate::domain::player::value_objects::PlayerName;
use crate::errors::{AppError, DomainError};
use crate::infrastructure::messaging::EventPublisher;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct JoinSessionCommand {
    pub session_code: String,
    pub display_name: String,
}

pub struct JoinSessionResult {
    pub player_id: Uuid,
    pub session_id: Uuid,
}

pub struct JoinSessionHandler<SR, PR, P> {
    session_repo: SR,
    player_repo: PR,
    event_publisher: P,
}

impl<SR, PR, P> JoinSessionHandler<SR, PR, P>
where
    SR: GameSessionRepository,
    PR: PlayerRepository,
    P: EventPublisher,
{
    pub fn new(session_repo: SR, player_repo: PR, event_publisher: P) -> Self {
        Self { session_repo, player_repo, event_publisher }
    }

    pub async fn handle(&self, cmd: JoinSessionCommand) -> Result<JoinSessionResult, AppError> {
        let code = GameCode::from_string(cmd.session_code)
            .map_err(|e| AppError::BadRequest(e))?;

        let session = self
            .session_repo
            .find_by_code(&code)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        if session.status != SessionStatus::Lobby {
            return Err(AppError::Domain(DomainError::SessionNotInLobby));
        }

        let name = PlayerName::new(cmd.display_name)
            .map_err(|e| AppError::BadRequest(e))?;

        let already_taken = self
            .player_repo
            .exists_in_session(session.id, name.value())
            .await?;

        if already_taken {
            return Err(AppError::Domain(DomainError::PlayerNameTaken));
        }

        let mut player = Player::join(
            session.id,
            code.value().to_string(),
            name,
        );

        let result = JoinSessionResult {
            player_id: player.id,
            session_id: session.id,
        };

        self.player_repo.save(&player).await?;

        let events = player.take_pending_events();
        for event in events {
            self.event_publisher.publish_player_event(event).await?;
        }

        Ok(result)
    }
}
