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

pub struct CloseRoundResult {
    pub correct_answer: String,
    pub has_ich_oder_du: bool,
    pub ich_oder_du_text: Option<String>,
}

pub async fn handle(
    session_code: &str,
    round_repo: &GameRoundRepository,
    state_repo: &GameStateRepository,
    pubsub: &RedisPubSub,
) -> Result<CloseRoundResult, AppError> {
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

    let now = Utc::now();
    round.closed_at = Some(now);

    let has_ich_oder_du = round.ich_oder_du_text.is_some();

    if has_ich_oder_du {
        // Move to ich_oder_du phase
        round.status = RoundStatus::IchOderDu;
        round_repo.update(&round).await?;

        // Update game state
        let updated_state = GameState {
            status: GameStatus::IchOderDu,
            updated_at: now,
            ..game_state
        };
        state_repo.upsert(&updated_state).await?;

        // Publish IchOderDuStarted
        let event = GameEvent::IchOderDuStarted {
            round_id,
            ich_oder_du_text: round.ich_oder_du_text.clone().unwrap_or_default(),
        };
        publish_to_both(pubsub, session_code, &event).await?;

        // Also publish RoundClosed so scoring-service knows the correct answer
        let closed_event = GameEvent::RoundClosed {
            round_id,
            correct_answer: round.correct_answer.clone(),
            closed_at: now,
        };
        publish_to_both(pubsub, session_code, &closed_event).await?;

        Ok(CloseRoundResult {
            correct_answer: round.correct_answer,
            has_ich_oder_du: true,
            ich_oder_du_text: round.ich_oder_du_text,
        })
    } else {
        // No ich-oder-du: move directly to scored
        round.status = RoundStatus::Scored;
        round_repo.update(&round).await?;

        let updated_state = GameState {
            status: GameStatus::Question, // stays at question until next-question advances it
            updated_at: now,
            ..game_state
        };
        state_repo.upsert(&updated_state).await?;

        let event = GameEvent::RoundClosed {
            round_id,
            correct_answer: round.correct_answer.clone(),
            closed_at: now,
        };
        publish_to_both(pubsub, session_code, &event).await?;

        Ok(CloseRoundResult {
            correct_answer: round.correct_answer,
            has_ich_oder_du: false,
            ich_oder_du_text: None,
        })
    }
}
