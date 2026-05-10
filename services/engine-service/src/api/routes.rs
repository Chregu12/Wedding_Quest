use axum::routing::{get, post};
use rf_web::RouterBuilder;

use crate::infrastructure::AppState;

use super::handlers::game_handlers;

pub fn routes(state: AppState) -> axum::Router {
    RouterBuilder::new()
        .route("/health", get(health_check))
        .route("/games/:code/start", post(game_handlers::start_game))
        .route("/games/:code/state", get(game_handlers::get_state))
        .route("/games/:code/answer", post(game_handlers::submit_answer))
        .route("/games/:code/close-round", post(game_handlers::close_round))
        .route(
            "/games/:code/couple-answer",
            post(game_handlers::couple_answer),
        )
        .route(
            "/games/:code/next-question",
            post(game_handlers::next_question),
        )
        .with_tracing(true)
        .with_cors(true)
        .build()
        .layer(axum::Extension(state))
}

async fn health_check() -> &'static str {
    "OK"
}
