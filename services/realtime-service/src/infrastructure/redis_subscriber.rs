use anyhow::Result;
use rf_broadcast::RoomRegistry;
use rf_cache::RedisPubSub;
use std::sync::Arc;

pub struct RedisSubscriber {
    pubsub: RedisPubSub,
    registry: Arc<RoomRegistry>,
}

impl RedisSubscriber {
    pub async fn new(redis_url: &str, registry: Arc<RoomRegistry>) -> Result<Self> {
        let pubsub = RedisPubSub::new(redis_url)
            .await
            .map_err(|e| anyhow::anyhow!("Redis PubSub init: {e}"))?;
        Ok(Self { pubsub, registry })
    }

    pub async fn run(self) -> Result<()> {
        let mut rx = self.pubsub
            .psubscribe("wedding_quest:session:*")
            .await
            .map_err(|e| anyhow::anyhow!("Redis psubscribe: {e}"))?;

        tracing::info!("Redis subscriber listening on wedding_quest:session:*");

        while let Some(msg) = rx.recv().await {
            let room_key = msg.channel
                .strip_prefix("wedding_quest:session:")
                .unwrap_or(&msg.channel)
                .to_string();

            tracing::debug!("Room {room_key} <- {}", msg.payload);
            self.registry.broadcast(&room_key, &msg.payload);
        }

        tracing::warn!("Redis PubSub stream ended");
        Ok(())
    }
}
