pub mod messaging;
pub mod persistence;

use anyhow::Result;
use rf_cache::RedisPubSub;
use rf_orm::{DatabaseConfig, DatabaseManager};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub pubsub: Arc<RedisPubSub>,
}

impl AppState {
    pub async fn init() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".into());

        let db = DatabaseManager::connect(DatabaseConfig {
            url: database_url,
            max_connections: 10,
            min_connections: 1,
            ..Default::default()
        })
        .await?;

        let pubsub = RedisPubSub::new(&redis_url)
            .await
            .map_err(|e| anyhow::anyhow!("Redis PubSub init: {e}"))?;

        Ok(Self {
            db: Arc::new(db),
            pubsub: Arc::new(pubsub),
        })
    }
}
