use anyhow::Result;

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
                .unwrap_or_else(|_| "engine_service=debug,tower_http=debug".into()),
        )
        .init();

    let config = config::Config::from_env()?;
    let state = infrastructure::AppState::init(&config).await?;
    let app = api::routes::routes(state);

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    tracing::info!("engine-service listening on 0.0.0.0:{}", config.port);

    axum::serve(listener, app).await?;
    Ok(())
}
