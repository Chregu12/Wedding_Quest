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

    // For ich_oder_du questions: correct_answer is determined by couple agreement
    let is_ich_oder_du = round.question_type == "ich_oder_du";
    if is_ich_oder_du {
        // Override correct_answer based on couple's answer
        let effective_correct = match &round.couple_answer {
            Some(ca) if ca != "uneinig" => ca.clone(),
            _ => String::new(), // No correct answer if couple disagrees or hasn't answered
        };
        round.correct_answer = effective_correct.clone();
    }

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
        // No paired ich-oder-du: move directly to scored
        round.status = RoundStatus::Scored;
        round_repo.update(&round).await?;

        let updated_state = GameState {
            status: GameStatus::Question,
            updated_at: now,
            ..game_state
        };
        state_repo.upsert(&updated_state).await?;

        let effective_answer = round.correct_answer.clone();
        let event = GameEvent::RoundClosed {
            round_id,
            correct_answer: effective_answer.clone(),
            closed_at: now,
        };
        publish_to_both(pubsub, session_code, &event).await?;

        Ok(CloseRoundResult {
            correct_answer: effective_answer,
            has_ich_oder_du: false,
            ich_oder_du_text: None,
        })
    }
}
