use anyhow::Result;
use std::env;

pub struct Config {
    pub port: u16,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "3006".into())
                .parse()?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
        })
    }
}
