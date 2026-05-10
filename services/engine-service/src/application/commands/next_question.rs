use chrono::Utc;
use uuid::Uuid;

use crate::domain::game::{
    entity::{GameRound, GameState, GameStatus, RoundStatus},
    events::GameEvent,
};
use crate::errors::AppError;
use crate::infrastructure::{
    persistence::{
        game_round_repository::GameRoundRepository, game_state_repository::GameStateRepository,
    },
    session_client::SessionClient,
};
use rf_cache::RedisPubSub;

use super::start_game::publish_to_both;

pub enum NextQuestionResult {
    NextRound {
        round_id: Uuid,
        question_text: String,
        option_a: Option<String>,
        option_b: Option<String>,
        option_c: Option<String>,
        option_d: Option<String>,
        round_number: i32,
        total_questions: i32,
    },
    GameOver,
}

pub async fn handle(
    session_code: &str,
    session_client: &SessionClient,
    round_repo: &GameRoundRepository,
    state_repo: &GameStateRepository,
    pubsub: &RedisPubSub,
) -> Result<NextQuestionResult, AppError> {
    let game_state = state_repo
        .find(session_code)
        .await?
        .ok_or_else(|| AppError::NotFound("Game state not found".into()))?;

    let next_number = game_state.current_round_number + 1;

    if next_number > game_state.total_questions {
        // Game over
        let now = Utc::now();
        let updated_state = GameState {
            status: GameStatus::Finished,
            current_round_id: None,
            current_round_number: game_state.current_round_number,
            updated_at: now,
            ..game_state
        };
        state_repo.upsert(&updated_state).await?;

        let event = GameEvent::GameEnded {
            session_code: session_code.to_string(),
        };
        publish_to_both(pubsub, session_code, &event).await?;

        return Ok(NextQuestionResult::GameOver);
    }

    // Load questions from session-service to find the next one
    let all_questions = session_client
        .get_questions(session_code)
        .await
        .map_err(|e| AppError::Internal(e))?;

    let mut guest_quiz: Vec<_> = all_questions
        .iter()
        .filter(|q| q.question_type == "guest_quiz")
        .collect();
    let mut ich_oder_du: Vec<_> = all_questions
        .iter()
        .filter(|q| q.question_type == "ich_oder_du")
        .collect();

    guest_quiz.sort_by_key(|q| q.order_index);
    ich_oder_du.sort_by_key(|q| q.order_index);

    // next_number is 1-based
    let idx = (next_number - 1) as usize;
    let next_q = guest_quiz
        .get(idx)
        .ok_or_else(|| AppError::BadRequest("No question at that index".into()))?;
    let paired_iod = ich_oder_du.get(idx).copied();

    let now = Utc::now();
    let round_id = Uuid::new_v4();

    let next_round = GameRound {
        id: round_id,
        session_code: session_code.to_string(),
        question_id: next_q.id,
        question_type: next_q.question_type.clone(),
        question_text: next_q.text.clone(),
        option_a: next_q.option_a.clone(),
        option_b: next_q.option_b.clone(),
        option_c: next_q.option_c.clone(),
        option_d: next_q.option_d.clone(),
        correct_answer: next_q.correct_answer.clone(),
        ich_oder_du_id: paired_iod.map(|q| q.id),
        ich_oder_du_text: paired_iod.map(|q| q.text.clone()),
        ich_oder_du_correct: paired_iod.map(|q| q.correct_answer.clone()),
        couple_answer: None,
        status: RoundStatus::Active,
        round_number: next_number,
        started_at: now,
        closed_at: None,
    };

    round_repo.save(&next_round).await?;

    let updated_state = GameState {
        status: GameStatus::Question,
        current_round_id: Some(round_id),
        current_round_number: next_number,
        updated_at: now,
        ..game_state
    };
    state_repo.upsert(&updated_state).await?;

    let event = GameEvent::QuestionStarted {
        round_id,
        question_id: next_q.id,
        question_type: next_q.question_type.clone(),
        question_text: next_q.text.clone(),
        option_a: next_q.option_a.clone(),
        option_b: next_q.option_b.clone(),
        option_c: next_q.option_c.clone(),
        option_d: next_q.option_d.clone(),
        correct_answer: next_q.correct_answer.clone(),
        started_at: now,
        round_number: next_number,
    };
    publish_to_both(pubsub, session_code, &event).await?;

    Ok(NextQuestionResult::NextRound {
        round_id,
        question_text: next_q.text.clone(),
        option_a: next_q.option_a.clone(),
        option_b: next_q.option_b.clone(),
        option_c: next_q.option_c.clone(),
        option_d: next_q.option_d.clone(),
        round_number: next_number,
        total_questions: game_state.total_questions,
    })
}
