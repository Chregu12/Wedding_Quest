// Internal AppError for domain / application / infrastructure layers.
// API handlers map this to rf_core::AppError for HTTP responses.

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Cache error: {0}")]
    Cache(#[from] rf_cache::CacheError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
