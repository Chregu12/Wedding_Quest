use crate::domain::game_session::entity::GameSession;
use crate::domain::game_session::repository::GameSessionRepository;
use crate::domain::game_session::value_objects::GameCode;
use crate::errors::AppError;
use uuid::Uuid;

pub struct GetSessionByIdQuery {
    pub session_id: Uuid,
}

pub struct GetSessionByCodeQuery {
    pub code: String,
}

pub struct GetSessionQueryHandler<R> {
    session_repo: R,
}

impl<R: GameSessionRepository> GetSessionQueryHandler<R> {
    pub fn new(session_repo: R) -> Self {
        Self { session_repo }
    }

    pub async fn by_id(&self, query: GetSessionByIdQuery) -> Result<GameSession, AppError> {
        self.session_repo
            .find_by_id(query.session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))
    }

    pub async fn by_code(&self, query: GetSessionByCodeQuery) -> Result<GameSession, AppError> {
        let code = GameCode::from_string(query.code)
            .map_err(|e| AppError::BadRequest(e))?;
        self.session_repo
            .find_by_code(&code)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))
    }
}
