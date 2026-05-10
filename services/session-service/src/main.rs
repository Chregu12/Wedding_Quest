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
                .unwrap_or_else(|_| "session_service=debug,tower_http=debug".into()),
        )
        .init();

    let state = infrastructure::AppState::init().await?;
    let app = api::routes::routes(state);

    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "3002".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("session-service listening on 0.0.0.0:{port}");

    axum::serve(listener, app).await?;
    Ok(())
}
