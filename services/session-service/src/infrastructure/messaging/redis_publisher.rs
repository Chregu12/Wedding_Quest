use super::EventPublisher;
use crate::domain::game_session::events::SessionDomainEvent;
use crate::domain::player::events::PlayerDomainEvent;
use crate::errors::AppError;
use async_trait::async_trait;
use rf_cache::RedisPubSub;
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisEventPublisher {
    pubsub: Arc<RedisPubSub>,
}

impl RedisEventPublisher {
    pub fn new(pubsub: Arc<RedisPubSub>) -> Self {
        Self { pubsub }
    }
}

#[async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish_session_event(&self, event: SessionDomainEvent) -> Result<(), AppError> {
        let channel = event.channel();
        let payload = serde_json::to_string(&event).map_err(anyhow::Error::from)?;
        self.pubsub.publish(&channel, &payload).await?;
        tracing::debug!("Published session event to {channel}");
        Ok(())
    }

    async fn publish_player_event(&self, event: PlayerDomainEvent) -> Result<(), AppError> {
        let channel = event.channel();
        let payload = serde_json::to_string(&event).map_err(anyhow::Error::from)?;
        self.pubsub.publish(&channel, &payload).await?;
        tracing::debug!("Published player event to {channel}");
        Ok(())
    }
}
