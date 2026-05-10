use crate::infrastructure::AppState;
use axum::routing::{delete, get, post, put};
use rf_web::RouterBuilder;

use super::handlers::{player_handlers, question_handlers, session_handlers};

pub fn routes(state: AppState) -> axum::Router {
    RouterBuilder::new()
        .route("/health", get(health_check))
        .route("/sessions", post(session_handlers::create_session))
        .route("/sessions/:code", get(session_handlers::get_session))
        .route("/sessions/:code/start", post(session_handlers::start_session))
        .route("/sessions/:code/players", get(player_handlers::get_lobby))
        .route("/sessions/:code/join", post(player_handlers::join_session))
        .route(
            "/sessions/:code/questions",
            post(question_handlers::add_guest_quiz).get(question_handlers::list_questions),
        )
        .route(
            "/sessions/:code/questions/:question_id",
            delete(question_handlers::delete_question),
        )
        .route(
            "/sessions/:code/ich-oder-du",
            post(question_handlers::add_ich_oder_du),
        )
        .route(
            "/sessions/:code/config",
            put(question_handlers::update_score_config),
        )
        .with_tracing(true)
        .with_cors(true)
        .build()
        .layer(axum::Extension(state))
}

async fn health_check() -> &'static str {
    "OK"
}
