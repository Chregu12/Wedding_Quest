use anyhow::Result;
use rf_config::{AppConfig, ConfigLoader};
use std::env;

pub struct ServiceConfig {
    pub app: AppConfig,
    pub redis_url: String,
}

impl ServiceConfig {
    pub fn load() -> Result<Self> {
        let app = match ConfigLoader::new().load::<AppConfig>() {
            Ok(cfg) => cfg,
            Err(_) => AppConfig {
                server: rf_config::ServerConfig::default(),
                database: rf_config::DatabaseConfig::default(),
                auth: rf_config::AuthConfig::default(),
            },
        };

        Ok(Self {
            app,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
        })
    }
}
