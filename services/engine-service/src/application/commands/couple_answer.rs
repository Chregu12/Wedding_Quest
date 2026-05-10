use chrono::Utc;

use crate::domain::game::{
    entity::{GameState, GameStatus, RoundStatus},
    events::GameEvent,
};
use crate::errors::AppError;
use crate::infrastructure::persistence::{
    game_round_repository::GameRoundRepository, game_state_repository::GameStateRepository,
};
use rf_cache::RedisPubSub;

use super::start_game::publish_to_both;

pub async fn handle(
    session_code: &str,
    answer: String,
    round_repo: &GameRoundRepository,
    state_repo: &GameStateRepository,
    pubsub: &RedisPubSub,
) -> Result<(), AppError> {
    let game_state = state_repo
        .find(session_code)
        .await?
        .ok_or_else(|| AppError::NotFound("Game state not found".into()))?;

    let round_id = game_state
        .current_round_id
        .ok_or_else(|| AppError::BadRequest("No active round".into()))?;

    let mut round = round_repo
        .find_by_id(round_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Round not found".into()))?;

    // Store couple answer and move to scored
    round.couple_answer = Some(answer.clone());
    round.status = RoundStatus::Scored;
    round_repo.update(&round).await?;

    let now = Utc::now();
    let updated_state = GameState {
        status: GameStatus::Question,
        updated_at: now,
        ..game_state
    };
    state_repo.upsert(&updated_state).await?;

    let event = GameEvent::CoupleAnswered {
        round_id,
        couple_answer: answer,
    };
    publish_to_both(pubsub, session_code, &event).await?;

    Ok(())
}
