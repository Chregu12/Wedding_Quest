pub mod redis_publisher;

use crate::domain::game_session::events::SessionDomainEvent;
use crate::domain::player::events::PlayerDomainEvent;
use crate::errors::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_session_event(&self, event: SessionDomainEvent) -> Result<(), AppError>;
    async fn publish_player_event(&self, event: PlayerDomainEvent) -> Result<(), AppError>;
}
