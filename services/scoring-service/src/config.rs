pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub engine_service_url: String,
    pub session_service_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "3004".into())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            engine_service_url: std::env::var("ENGINE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".into()),
            session_service_url: std::env::var("SESSION_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".into()),
        })
    }
}
