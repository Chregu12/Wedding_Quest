use axum::{extract::Path, http::StatusCode, Extension, Json};
use rf_core::{AppError, AppResult};

use uuid::Uuid;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::{
    api::dto::game_dto::{
        CloseRoundResponse, CoupleAnswerRequest, CoupleIndividualAnswerRequest,
        CoupleIndividualAnswerResponse, GameStateResponse, NextQuestionResponse,
        RoundAnswerResponse, StartGameResponse, SubmitAnswerRequest, SubmitAnswerResponse,
    },
    application::commands::{
        close_round, couple_answer, next_question, reset_game, start_game, submit_answer,
    },
    infrastructure::{
        persistence::{
            game_round_repository::GameRoundRepository,
            game_state_repository::GameStateRepository,
            models::player_answer,
        },
        AppState,
    },
};

pub async fn start_game(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<StartGameResponse>> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let question_id = body.get("question_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Uuid>().ok());

    let result = start_game::handle(
        &code,
        &state.session_client,
        &round_repo,
        &state_repo,
        &state.pubsub,
        question_id,
    )
    .await
    .map_err(map_err)?;

    Ok(Json(StartGameResponse {
        round_id: result.round_id,
        question_text: result.question_text,
        option_a: result.option_a,
        option_b: result.option_b,
        option_c: result.option_c,
        option_d: result.option_d,
        correct_answer: result.correct_answer,
        round_number: result.round_number,
        total_questions: result.total_questions,
    }))
}

/// POST /games/:code/reset
/// Reset the game back to the lobby/waiting state so guests and the couple
/// stop seeing a stale question from a previous run before a new game starts.
pub async fn reset_game(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<StatusCode> {
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    reset_game::handle(&code, &state_repo)
        .await
        .map_err(map_err)?;

    Ok(StatusCode::OK)
}

pub async fn get_state(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<GameStateResponse>> {
    let state_repo = GameStateRepository::new(state.db.connection().clone());
    let round_repo = GameRoundRepository::new(state.db.connection().clone());

    let game_state = state_repo
        .find(&code)
        .await
        .map_err(map_err)?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("Game state for code {code}"),
        })?;

    // Load current round details if available
    let round = if let Some(rid) = game_state.current_round_id {
        round_repo.find_by_id(rid).await.map_err(map_err)?
    } else {
        None
    };

    Ok(Json(GameStateResponse {
        status: game_state.status.as_str().to_string(),
        current_round_id: game_state.current_round_id,
        current_round_number: game_state.current_round_number,
        total_questions: game_state.total_questions,
        question_type: round.as_ref().map(|r| r.question_type.clone()),
        question_text: round.as_ref().map(|r| r.question_text.clone()),
        option_a: round.as_ref().and_then(|r| r.option_a.clone()),
        option_b: round.as_ref().and_then(|r| r.option_b.clone()),
        option_c: round.as_ref().and_then(|r| r.option_c.clone()),
        option_d: round.as_ref().and_then(|r| r.option_d.clone()),
        started_at: round.as_ref().map(|r| r.started_at.to_rfc3339()),
    }))
}

pub async fn submit_answer(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<SubmitAnswerRequest>,
) -> AppResult<Json<SubmitAnswerResponse>> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let result = submit_answer::handle(
        submit_answer::SubmitAnswerCommand {
            session_code: code,
            player_id: req.player_id,
            player_name: req.player_name,
            answer: req.answer,
            couple: req.couple,
        },
        &round_repo,
        &state_repo,
        state.db.connection(),
    )
    .await
    .map_err(map_err)?;

    Ok(Json(SubmitAnswerResponse {
        accepted: result.accepted,
        is_correct: result.is_correct,
    }))
}

pub async fn close_round(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<CloseRoundResponse>> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let result = close_round::handle(&code, &round_repo, &state_repo, &state.pubsub)
        .await
        .map_err(map_err)?;

    Ok(Json(CloseRoundResponse {
        correct_answer: result.correct_answer,
        has_ich_oder_du: result.has_ich_oder_du,
        ich_oder_du_text: result.ich_oder_du_text,
    }))
}

pub async fn couple_answer(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<CoupleAnswerRequest>,
) -> AppResult<StatusCode> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    couple_answer::handle(&code, req.answer, &round_repo, &state_repo, &state.pubsub)
        .await
        .map_err(map_err)?;

    Ok(StatusCode::OK)
}

pub async fn next_question(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<Option<NextQuestionResponse>>)> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let question_id = body.get("question_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Uuid>().ok());

    let result = next_question::handle(
        &code,
        &state.session_client,
        &round_repo,
        &state_repo,
        &state.pubsub,
        question_id,
    )
    .await
    .map_err(map_err)?;

    match result {
        next_question::NextQuestionResult::NextRound {
            round_id,
            question_text,
            option_a,
            option_b,
            option_c,
            option_d,
            correct_answer,
            round_number,
            total_questions,
        } => Ok((
            StatusCode::OK,
            Json(Some(NextQuestionResponse {
                round_id,
                question_text,
                option_a,
                option_b,
                option_c,
                option_d,
                correct_answer,
                round_number,
                total_questions,
            })),
        )),
        next_question::NextQuestionResult::GameOver => {
            Ok((StatusCode::NO_CONTENT, Json(None)))
        }
    }
}

pub async fn get_round_answers(
    Extension(state): Extension<AppState>,
    Path((_code, round_id)): Path<(String, Uuid)>,
) -> AppResult<Json<Vec<RoundAnswerResponse>>> {
    use crate::infrastructure::persistence::models::game_round;

    let db = state.db.connection().clone();

    // Get the round to know the question_type
    let round = game_round::Entity::find_by_id(round_id)
        .one(&db).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
    let question_type = round.map(|r| r.question_type).unwrap_or_default();

    let answers = player_answer::Entity::find()
        .filter(player_answer::Column::RoundId.eq(round_id))
        .all(&db)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let response: Vec<RoundAnswerResponse> = answers
        .into_iter()
        .map(|a| RoundAnswerResponse {
            player_id: a.player_id,
            player_name: a.player_name,
            answer: a.answer,
            is_correct: a.is_correct,
            time_taken_seconds: a.time_taken_seconds.try_into().unwrap_or(0.0),
            question_type: question_type.clone(),
        })
        .collect();

    Ok(Json(response))
}

/// POST /games/:code/couple-individual-answer
/// Each couple member submits their answer independently.
/// When both have answered, publishes CoupleAnswered if they agree, or a "disagreed" event.
pub async fn couple_individual_answer(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<CoupleIndividualAnswerRequest>,
) -> AppResult<Json<CoupleIndividualAnswerResponse>> {
    use sea_orm::ActiveValue::Set;
    use crate::infrastructure::persistence::models::game_round;

    let state_repo = GameStateRepository::new(state.db.connection().clone());
    let game_state = state_repo.find(&code).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound { resource: "Game state".into() })?;

    let round_id = game_state.current_round_id
        .ok_or_else(|| AppError::BadRequest { message: "No active round".into() })?;

    let db = state.db.connection().clone();
    let round = game_round::Entity::find_by_id(round_id)
        .one(&db).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound { resource: "Round".into() })?;

    // Update the appropriate column
    let mut active: game_round::ActiveModel = round.clone().into();
    if req.person == "a" {
        active.couple_answer_a = Set(Some(req.answer.clone()));
    } else {
        active.couple_answer_b = Set(Some(req.answer.clone()));
    }

    use sea_orm::ActiveModelTrait;
    let updated = active.update(&db).await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    let answer_a = updated.couple_answer_a.clone();
    let answer_b = updated.couple_answer_b.clone();

    if let (Some(a), Some(b)) = (&answer_a, &answer_b) {
        let agree = a == b;
        let final_answer = if agree { a.clone() } else { String::new() };

        // Store combined answer
        let mut final_active: game_round::ActiveModel = updated.into();
        final_active.couple_answer = Set(Some(if agree { a.clone() } else { "uneinig".to_string() }));
        final_active.status = Set("scored".to_string());
        final_active.update(&db).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

        // Publish CoupleAnswered event
        use crate::domain::game::events::GameEvent;
        use super::super::super::application::commands::start_game::publish_to_both;
        let event = GameEvent::CoupleAnswered {
            round_id,
            couple_answer: if agree { a.clone() } else { "uneinig".to_string() },
            answer_a: a.clone(),
            answer_b: b.clone(),
        };
        publish_to_both(&state.pubsub, &code, &event).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

        Ok(Json(CoupleIndividualAnswerResponse {
            both_answered: true,
            agree: Some(agree),
            final_answer: Some(final_answer),
        }))
    } else {
        Ok(Json(CoupleIndividualAnswerResponse {
            both_answered: false,
            agree: None,
            final_answer: None,
        }))
    }
}

fn map_err(e: crate::errors::AppError) -> AppError {
    match e {
        crate::errors::AppError::NotFound(msg) => AppError::NotFound { resource: msg },
        crate::errors::AppError::BadRequest(msg) => AppError::BadRequest { message: msg },
        crate::errors::AppError::Conflict(msg) => AppError::Conflict { message: msg },
        other => AppError::Internal(anyhow::anyhow!("{}", other)),
    }
}
