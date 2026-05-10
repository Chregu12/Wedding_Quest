use super::entity::Player;
use crate::errors::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PlayerRepository: Send + Sync {
    async fn save(&self, player: &Player) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>, AppError>;
    async fn find_by_session(&self, session_id: Uuid) -> Result<Vec<Player>, AppError>;
    async fn exists_in_session(&self, session_id: Uuid, name: &str) -> Result<bool, AppError>;
    async fn update(&self, player: &Player) -> Result<(), AppError>;
}
