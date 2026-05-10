use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use uuid::Uuid;

use crate::domain::game::entity::RoundStatus;
use crate::errors::AppError;
use crate::infrastructure::persistence::{
    game_round_repository::GameRoundRepository,
    game_state_repository::GameStateRepository,
    models::player_answer::ActiveModel as PlayerAnswerActiveModel,
};

pub struct SubmitAnswerCommand {
    pub session_code: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub answer: String,
}

pub struct SubmitAnswerResult {
    pub accepted: bool,
    pub is_correct: bool,
}

pub async fn handle(
    cmd: SubmitAnswerCommand,
    round_repo: &GameRoundRepository,
    state_repo: &GameStateRepository,
    db: &DatabaseConnection,
) -> Result<SubmitAnswerResult, AppError> {
    // Get current game state
    let game_state = state_repo
        .find(&cmd.session_code)
        .await?
        .ok_or_else(|| AppError::NotFound("Game state not found".into()))?;

    let round_id = game_state
        .current_round_id
        .ok_or_else(|| AppError::BadRequest("No active round".into()))?;

    let round = round_repo
        .find_by_id(round_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Round not found".into()))?;

    // Only accept answers when round is active
    if round.status != RoundStatus::Active {
        return Ok(SubmitAnswerResult {
            accepted: false,
            is_correct: false,
        });
    }

    let is_correct = round.correct_answer.to_uppercase() == cmd.answer.to_uppercase();
    let now = Utc::now();
    let time_taken = (now - round.started_at).num_seconds().max(0) as f64;

    // Upsert: if player already answered, ignore (unique index enforces once per round)
    let answer_model = PlayerAnswerActiveModel {
        id: Set(Uuid::new_v4()),
        round_id: Set(round_id),
        player_id: Set(cmd.player_id),
        player_name: Set(cmd.player_name.clone()),
        answer: Set(cmd.answer.clone()),
        is_correct: Set(is_correct),
        answered_at: Set(now.fixed_offset()),
        time_taken_seconds: Set(rust_decimal::Decimal::from_f64_retain(time_taken)
            .unwrap_or_default()),
    };

    // ON CONFLICT DO NOTHING via insert — if a unique-index violation occurs we treat it as accepted
    match answer_model.insert(db).await {
        Ok(_) => {}
        Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(ref e)))
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") =>
        {
            // Player already answered — silently ignore
        }
        Err(e) => return Err(AppError::Database(e)),
    }

    Ok(SubmitAnswerResult {
        accepted: true,
        is_correct,
    })
}
