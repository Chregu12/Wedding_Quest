use crate::domain::game_session::value_objects::GameCode;
use crate::domain::game_session::repository::GameSessionRepository;
use crate::domain::player::entity::Player;
use crate::domain::player::repository::PlayerRepository;
use crate::errors::AppError;

pub struct GetLobbyQuery {
    pub session_code: String,
}

pub struct LobbyView {
    pub session_code: String,
    pub person_a_name: String,
    pub person_b_name: String,
    pub players: Vec<Player>,
}

pub struct GetLobbyHandler<SR, PR> {
    session_repo: SR,
    player_repo: PR,
}

impl<SR, PR> GetLobbyHandler<SR, PR>
where
    SR: GameSessionRepository,
    PR: PlayerRepository,
{
    pub fn new(session_repo: SR, player_repo: PR) -> Self {
        Self { session_repo, player_repo }
    }

    pub async fn handle(&self, query: GetLobbyQuery) -> Result<LobbyView, AppError> {
        let code = GameCode::from_string(query.session_code)
            .map_err(|e| AppError::BadRequest(e))?;

        let session = self
            .session_repo
            .find_by_code(&code)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        let players = self.player_repo.find_by_session(session.id).await?;

        Ok(LobbyView {
            session_code: code.value().to_string(),
            person_a_name: session.person_a_name,
            person_b_name: session.person_b_name,
            players,
        })
    }
}
