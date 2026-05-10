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

pub struct StartGameResult {
    pub round_id: Uuid,
    pub question_text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub round_number: i32,
    pub total_questions: i32,
}

pub async fn handle(
    session_code: &str,
    session_client: &SessionClient,
    round_repo: &GameRoundRepository,
    state_repo: &GameStateRepository,
    pubsub: &RedisPubSub,
) -> Result<StartGameResult, AppError> {
    // Fetch all questions from session-service
    let all_questions = session_client
        .get_questions(session_code)
        .await
        .map_err(|e| AppError::Internal(e))?;

    // Separate and sort by order_index
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

    if guest_quiz.is_empty() {
        return Err(AppError::BadRequest(
            "No guest_quiz questions found for this session".into(),
        ));
    }

    let total_questions = guest_quiz.len() as i32;

    // Build first round — pair index 0 with ich_oder_du index 0 (if available)
    let first_q = guest_quiz[0];
    let paired_iod = ich_oder_du.get(0).copied();

    let now = Utc::now();
    let round_id = Uuid::new_v4();

    let first_round = GameRound {
        id: round_id,
        session_code: session_code.to_string(),
        question_id: first_q.id,
        question_type: first_q.question_type.clone(),
        question_text: first_q.text.clone(),
        option_a: first_q.option_a.clone(),
        option_b: first_q.option_b.clone(),
        option_c: first_q.option_c.clone(),
        option_d: first_q.option_d.clone(),
        correct_answer: first_q.correct_answer.clone(),
        ich_oder_du_id: paired_iod.map(|q| q.id),
        ich_oder_du_text: paired_iod.map(|q| q.text.clone()),
        ich_oder_du_correct: paired_iod.map(|q| q.correct_answer.clone()),
        couple_answer: None,
        status: RoundStatus::Active,
        round_number: 1,
        started_at: now,
        closed_at: None,
    };

    round_repo.save(&first_round).await?;

    // Upsert game_state
    let game_state = GameState {
        session_code: session_code.to_string(),
        status: GameStatus::Question,
        current_round_id: Some(round_id),
        current_round_number: 1,
        total_questions,
        updated_at: now,
    };
    state_repo.upsert(&game_state).await?;

    // Publish event to both channels
    let event = GameEvent::QuestionStarted {
        round_id,
        question_id: first_q.id,
        question_type: first_q.question_type.clone(),
        question_text: first_q.text.clone(),
        option_a: first_q.option_a.clone(),
        option_b: first_q.option_b.clone(),
        option_c: first_q.option_c.clone(),
        option_d: first_q.option_d.clone(),
        correct_answer: first_q.correct_answer.clone(),
        started_at: now,
        round_number: 1,
    };
    publish_to_both(pubsub, session_code, &event).await?;

    Ok(StartGameResult {
        round_id,
        question_text: first_q.text.clone(),
        option_a: first_q.option_a.clone(),
        option_b: first_q.option_b.clone(),
        option_c: first_q.option_c.clone(),
        option_d: first_q.option_d.clone(),
        round_number: 1,
        total_questions,
    })
}

pub async fn publish_to_both(
    pubsub: &RedisPubSub,
    session_code: &str,
    event: &GameEvent,
) -> Result<(), AppError> {
    let json = event.to_json();
    pubsub
        .publish(
            &format!("wedding_quest:game:{}", session_code),
            &json,
        )
        .await
        .map_err(|e| AppError::Cache(e))?;
    pubsub
        .publish(
            &format!("wedding_quest:session:{}", session_code),
            &json,
        )
        .await
        .map_err(|e| AppError::Cache(e))?;
    Ok(())
}
