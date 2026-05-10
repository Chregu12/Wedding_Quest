use axum::{extract::Path, http::StatusCode, Extension, Json};
use rf_core::{AppError, AppResult};

use crate::{
    api::dto::game_dto::{
        CloseRoundResponse, CoupleAnswerRequest, GameStateResponse, NextQuestionResponse,
        StartGameResponse, SubmitAnswerRequest, SubmitAnswerResponse,
    },
    application::commands::{
        close_round, couple_answer, next_question, start_game, submit_answer,
    },
    infrastructure::{
        persistence::{
            game_round_repository::GameRoundRepository,
            game_state_repository::GameStateRepository,
        },
        AppState,
    },
};

pub async fn start_game(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<StartGameResponse>> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let result = start_game::handle(
        &code,
        &state.session_client,
        &round_repo,
        &state_repo,
        &state.pubsub,
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
        round_number: result.round_number,
        total_questions: result.total_questions,
    }))
}

pub async fn get_state(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<GameStateResponse>> {
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let game_state = state_repo
        .find(&code)
        .await
        .map_err(map_err)?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("Game state for code {code}"),
        })?;

    Ok(Json(GameStateResponse {
        status: game_state.status.as_str().to_string(),
        current_round_id: game_state.current_round_id,
        current_round_number: game_state.current_round_number,
        total_questions: game_state.total_questions,
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
) -> AppResult<(StatusCode, Json<Option<NextQuestionResponse>>)> {
    let round_repo = GameRoundRepository::new(state.db.connection().clone());
    let state_repo = GameStateRepository::new(state.db.connection().clone());

    let result = next_question::handle(
        &code,
        &state.session_client,
        &round_repo,
        &state_repo,
        &state.pubsub,
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
                round_number,
                total_questions,
            })),
        )),
        next_question::NextQuestionResult::GameOver => {
            Ok((StatusCode::NO_CONTENT, Json(None)))
        }
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
