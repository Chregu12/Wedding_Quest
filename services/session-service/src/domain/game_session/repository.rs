use super::entity::GameSession;
use super::value_objects::GameCode;
use crate::errors::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn save(&self, session: &GameSession) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GameSession>, AppError>;
    async fn find_by_code(&self, code: &GameCode) -> Result<Option<GameSession>, AppError>;
    async fn update(&self, session: &GameSession) -> Result<(), AppError>;
}
