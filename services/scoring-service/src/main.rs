use anyhow::Result;
use std::sync::Arc;

mod api;
mod application;
mod config;
mod domain;
mod errors;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scoring_service=debug,tower_http=debug".into()),
        )
        .init();

    let cfg = config::Config::from_env()?;
    let state = infrastructure::AppState::init(&cfg).await?;

    // Spawn event listener in a dedicated task.
    // It needs its own RedisPubSub connection for subscribing (subscribe/psubscribe
    // cannot share a connection with publish operations).
    let sub_pubsub = rf_cache::RedisPubSub::new(&cfg.redis_url)
        .await
        .map_err(|e| anyhow::anyhow!("Redis subscriber init: {e}"))?;

    let db_clone = Arc::clone(&state.db);
    let pub_pubsub = Arc::clone(&state.pubsub);
    let engine = Arc::clone(&state.engine_client);
    let session = Arc::clone(&state.session_client);

    tokio::spawn(async move {
        infrastructure::event_listener::run(
            Arc::new(sub_pubsub),
            db_clone,
            engine,
            session,
            pub_pubsub,
        )
        .await;
    });

    let app = api::routes::routes(state);

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port)).await?;
    tracing::info!("scoring-service listening on 0.0.0.0:{}", cfg.port);

    axum::serve(listener, app).await?;
    Ok(())
}
