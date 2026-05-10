use crate::infrastructure::AppState;
use axum::routing::{get, post};
use rf_web::RouterBuilder;

use super::handlers::{player_handlers, session_handlers};

pub fn routes(state: AppState) -> axum::Router {
    RouterBuilder::new()
        .route("/health", get(health_check))
        .route("/sessions", post(session_handlers::create_session))
        .route("/sessions/:code", get(session_handlers::get_session))
        .route("/sessions/:code/start", post(session_handlers::start_session))
        .route("/sessions/:code/players", get(player_handlers::get_lobby))
        .route("/sessions/:code/join", post(player_handlers::join_session))
        .with_tracing(true)
        .with_cors(true)
        .build()
        .layer(axum::Extension(state))
}

async fn health_check() -> &'static str {
    "OK"
}
