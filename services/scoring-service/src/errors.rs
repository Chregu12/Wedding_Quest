/// Internal AppError — used by domain, application, and infrastructure layers.
/// API handlers map this to rf_core::AppError for HTTP responses.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Cache error: {0}")]
    Cache(#[from] rf_cache::CacheError),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
