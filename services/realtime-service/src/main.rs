use anyhow::Result;
use rf_broadcast::RoomRegistry;
use std::sync::Arc;

mod api;
mod config;
mod domain;
mod infrastructure;

use infrastructure::redis_subscriber::RedisSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "realtime_service=debug,tower_http=debug".into()),
        )
        .init();

    let config = config::Config::from_env()?;
    let registry = Arc::new(RoomRegistry::new());

    let subscriber = RedisSubscriber::new(&config.redis_url, Arc::clone(&registry)).await?;
    tokio::spawn(async move {
        if let Err(e) = subscriber.run().await {
            tracing::error!("Redis subscriber error: {e}");
        }
    });

    let app = api::routes::routes(Arc::clone(&registry));

    tracing::info!("realtime-service listening on 0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
