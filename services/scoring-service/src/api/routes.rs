use axum::routing::get;
use rf_web::RouterBuilder;

use crate::infrastructure::AppState;

use super::handlers::score_handlers;

pub fn routes(state: AppState) -> axum::Router {
    RouterBuilder::new()
        .route("/health", get(health_check))
        .route("/scores/:code", get(score_handlers::get_leaderboard))
        .route("/scores/:code/config", get(score_handlers::get_score_config))
        .with_tracing(true)
        .with_cors(true)
        .build()
        .layer(axum::Extension(state))
}

async fn health_check() -> &'static str {
    "OK"
}
