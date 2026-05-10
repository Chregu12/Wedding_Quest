use crate::{
    api::dto::session_dto::{CreateSessionRequest, CreateSessionResponse, SessionResponse},
    application::commands::{
        create_session::{CreateSessionCommand, CreateSessionHandler},
        start_session::{StartSessionCommand, StartSessionHandler},
    },
    application::queries::get_session::{GetSessionByCodeQuery, GetSessionQueryHandler},
    domain::game_session::{
        repository::GameSessionRepository,
        value_objects::GameCode,
    },
    infrastructure::{
        messaging::redis_publisher::RedisEventPublisher,
        persistence::game_session_repository::SeaOrmGameSessionRepository,
        AppState,
    },
};
use std::sync::Arc;
use axum::{
    extract::Path,
    http::StatusCode,
    Extension, Json,
};
use rf_core::AppError;
use rf_core::AppResult;

pub async fn create_session(
    Extension(state): Extension<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<(StatusCode, Json<CreateSessionResponse>)> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let event_publisher = RedisEventPublisher::new(Arc::clone(&state.pubsub));
    let handler = CreateSessionHandler::new(session_repo, event_publisher);

    let result = handler
        .handle(CreateSessionCommand {
            person_a_name: req.person_a_name,
            person_b_name: req.person_b_name,
            host_name: req.host_name,
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse {
            session_id: result.session_id,
            code: result.code,
        }),
    ))
}

pub async fn get_session(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<SessionResponse>> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let handler = GetSessionQueryHandler::new(session_repo);
    let session = handler
        .by_code(GetSessionByCodeQuery { code })
        .await
        .map_err(|e| match e {
            crate::errors::AppError::NotFound(msg) => AppError::NotFound { resource: msg },
            other => AppError::Internal(anyhow::anyhow!("{}", other)),
        })?;

    Ok(Json(SessionResponse {
        id: session.id,
        code: session.code.value().to_string(),
        status: session.status.as_str().to_string(),
        person_a_name: session.person_a_name,
        person_b_name: session.person_b_name,
        started_at: session.started_at,
        created_at: session.created_at,
    }))
}

pub async fn start_session(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<StatusCode> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let event_publisher = RedisEventPublisher::new(Arc::clone(&state.pubsub));

    let code_vo = GameCode::from_string(code)
        .map_err(|e| AppError::BadRequest { message: e })?;

    let session = session_repo
        .find_by_code(&code_vo)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| AppError::NotFound { resource: "Session".into() })?;

    let handler = StartSessionHandler::new(session_repo, event_publisher);
    handler
        .handle(StartSessionCommand { session_id: session.id })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
