pub mod engine_client;
pub mod event_listener;
pub mod persistence;
pub mod session_client;

use std::sync::Arc;

use anyhow::Result;
use rf_cache::RedisPubSub;
use rf_orm::{DatabaseConfig, DatabaseManager};

use crate::config::Config;

use engine_client::EngineClient;
use session_client::SessionClient;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub pubsub: Arc<RedisPubSub>,
    pub engine_client: Arc<EngineClient>,
    pub session_client: Arc<SessionClient>,
}

impl AppState {
    pub async fn init(config: &Config) -> Result<Self> {
        let db = DatabaseManager::connect(DatabaseConfig {
            url: config.database_url.clone(),
            max_connections: 10,
            min_connections: 1,
            ..Default::default()
        })
        .await?;

        let pubsub = RedisPubSub::new(&config.redis_url)
            .await
            .map_err(|e| anyhow::anyhow!("Redis PubSub init: {e}"))?;

        let engine_client = EngineClient::new(config.engine_service_url.clone());
        let session_client = SessionClient::new(config.session_service_url.clone());

        Ok(Self {
            db: Arc::new(db),
            pubsub: Arc::new(pubsub),
            engine_client: Arc::new(engine_client),
            session_client: Arc::new(session_client),
        })
    }
}
