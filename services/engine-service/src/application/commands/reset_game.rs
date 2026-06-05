use chrono::Utc;

use crate::domain::game::entity::{GameState, GameStatus};
use crate::errors::AppError;
use crate::infrastructure::persistence::game_state_repository::GameStateRepository;

/// Reset the game state for a session back to the lobby/waiting state.
///
/// Called when the moderator opens the game console for a (new) game so that
/// guests and the couple do not keep seeing a stale question from a previous
/// run on their waiting screen. The next `start_game` creates a fresh round.
pub async fn handle(
    session_code: &str,
    state_repo: &GameStateRepository,
) -> Result<(), AppError> {
    let now = Utc::now();

    // Preserve total_questions if a state row already exists.
    let total_questions = state_repo
        .find(session_code)
        .await?
        .map(|s| s.total_questions)
        .unwrap_or(0);

    let reset_state = GameState {
        session_code: session_code.to_string(),
        status: GameStatus::Waiting,
        current_round_id: None,
        current_round_number: 0,
        total_questions,
        updated_at: now,
    };
    state_repo.upsert(&reset_state).await?;

    Ok(())
}
