use crate::{
    api::dto::player_dto::{JoinSessionRequest, JoinSessionResponse, LobbyResponse, PlayerResponse},
    application::commands::join_session::{JoinSessionCommand, JoinSessionHandler},
    application::queries::get_lobby::{GetLobbyHandler, GetLobbyQuery},
    infrastructure::{
        messaging::redis_publisher::RedisEventPublisher,
        persistence::{
            game_session_repository::SeaOrmGameSessionRepository,
            player_repository::SeaOrmPlayerRepository,
        },
        AppState,
    },
};
use std::sync::Arc;
use axum::{extract::Path, http::StatusCode, Extension, Json};
use rf_core::AppError;
use rf_core::AppResult;

pub async fn join_session(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<JoinSessionRequest>,
) -> AppResult<(StatusCode, Json<JoinSessionResponse>)> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let player_repo = SeaOrmPlayerRepository::new(state.db.connection().clone());
    let event_publisher = RedisEventPublisher::new(Arc::clone(&state.pubsub));
    let handler = JoinSessionHandler::new(session_repo, player_repo, event_publisher);

    let result = handler
        .handle(JoinSessionCommand {
            session_code: code,
            display_name: req.display_name,
        })
        .await
        .map_err(|e| match e {
            crate::errors::AppError::NotFound(msg) => AppError::NotFound { resource: msg },
            crate::errors::AppError::BadRequest(msg) => AppError::BadRequest { message: msg },
            crate::errors::AppError::Domain(d) => AppError::BadRequest { message: d.to_string() },
            other => AppError::Internal(anyhow::anyhow!("{}", other)),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(JoinSessionResponse {
            player_id: result.player_id,
            session_id: result.session_id,
        }),
    ))
}

pub async fn get_lobby(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<LobbyResponse>> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let player_repo = SeaOrmPlayerRepository::new(state.db.connection().clone());
    let handler = GetLobbyHandler::new(session_repo, player_repo);

    let lobby = handler
        .handle(GetLobbyQuery { session_code: code })
        .await
        .map_err(|e| match e {
            crate::errors::AppError::NotFound(msg) => AppError::NotFound { resource: msg },
            other => AppError::Internal(anyhow::anyhow!("{}", other)),
        })?;

    let players = lobby
        .players
        .into_iter()
        .map(|p| PlayerResponse {
            id: p.id,
            display_name: p.display_name.value().to_string(),
            avatar: p.avatar,
            total_score: p.total_score,
            is_connected: p.is_connected,
            joined_at: p.joined_at,
        })
        .collect();

    Ok(Json(LobbyResponse {
        session_code: lobby.session_code,
        person_a_name: lobby.person_a_name,
        person_b_name: lobby.person_b_name,
        players,
    }))
}
