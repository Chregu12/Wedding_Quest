use axum::{extract::Path, Extension, Json};
use rf_core::{AppError, AppResult};

use crate::{
    api::dto::score_dto::{LeaderboardResponse, PlayerScoreResponse},
    infrastructure::{
        persistence::score_repository::ScoreRepository,
        session_client::ScoreConfigDto,
        AppState,
    },
};

/// GET /scores/:code
///
/// Returns the current leaderboard for the session, sorted by total_score DESC.
pub async fn get_leaderboard(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<LeaderboardResponse>> {
    let repo = ScoreRepository::new(state.db.connection().clone());

    let scores = repo
        .find_by_session(&code)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let player_scores: Vec<PlayerScoreResponse> = scores
        .iter()
        .enumerate()
        .map(|(i, ps)| {
            let rank = if i == 0 {
                1
            } else if ps.total_score == scores[i - 1].total_score {
                let mut r = i + 1;
                for j in (0..i).rev() {
                    if scores[j].total_score == ps.total_score { r = j + 1; } else { break; }
                }
                r
            } else {
                i + 1
            };
            PlayerScoreResponse {
                player_id: ps.player_id,
                player_name: ps.player_name.clone(),
                total_score: ps.total_score,
                rounds_played: ps.rounds_played,
                last_round_score: ps.last_round_score,
                rank,
            }
        })
        .collect();

    Ok(Json(LeaderboardResponse {
        session_code: code,
        scores: player_scores,
    }))
}

/// DELETE /scores/:code
///
/// Resets all scores for a session (used when restarting a game).
pub async fn reset_scores(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<()> {
    let repo = ScoreRepository::new(state.db.connection().clone());
    repo.delete_by_session(&code)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
    Ok(())
}

/// GET /scores/:code/config
///
/// Proxies the score config from session-service so the Angular frontend can
/// read the current multiplier settings without knowing the session-service URL.
pub async fn get_score_config(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<ScoreConfigDto>> {
    let config = state
        .session_client
        .get_score_config(&code)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("session-service error: {e}")))?;

    Ok(Json(config))
}
