use rf_broadcast::RoomRegistry;
use axum::{routing::get, Extension};
use rf_web::RouterBuilder;
use std::sync::Arc;

use super::ws_handler;

pub fn routes(registry: Arc<RoomRegistry>) -> axum::Router {
    RouterBuilder::new()
        .route("/health", get(health_check))
        .route("/ws/:session_id", get(ws_handler::ws_upgrade))
        .with_tracing(true)
        .with_cors(true)
        .build()
        .layer(Extension(registry))
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "realtime-service"
    }))
}
