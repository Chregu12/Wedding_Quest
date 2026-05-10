use axum::{extract::Path, http::StatusCode, Extension, Json};
use rf_core::{AppError, AppResult};

use crate::{
    api::dto::question_dto::{
        AddGuestQuizRequest, AddIchOderDuRequest, QuestionResponse, ScoreConfigResponse,
        UpdateScoreConfigRequest,
    },
    domain::{
        game_session::{repository::GameSessionRepository, value_objects::GameCode},
        question::{
            entity::Question,
            repository::QuestionRepository,
            value_objects::{AnswerOption, CoupleAnswer},
        },
        score_config::entity::ScoreConfig,
    },
    infrastructure::{
        persistence::{
            game_session_repository::SeaOrmGameSessionRepository,
            question_repository::SeaOrmQuestionRepository,
            score_config_repository::SeaOrmScoreConfigRepository,
        },
        AppState,
    },
};

pub async fn add_guest_quiz(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<AddGuestQuizRequest>,
) -> AppResult<(StatusCode, Json<QuestionResponse>)> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let code_vo =
        GameCode::from_string(code).map_err(|e| AppError::BadRequest { message: e })?;
    let session = session_repo
        .find_by_code(&code_vo)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound {
            resource: "Session".into(),
        })?;

    let correct = AnswerOption::from_str(&req.correct_answer)
        .map_err(|e| AppError::BadRequest { message: e.to_string() })?;

    let question = Question::create_guest_quiz(
        session.id,
        req.text,
        req.option_a,
        req.option_b,
        req.option_c,
        req.option_d,
        correct,
        req.order_index.unwrap_or(0),
        req.points.unwrap_or(100),
    );

    let question_repo = SeaOrmQuestionRepository::new(state.db.connection().clone());
    question_repo
        .save(&question)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    Ok((StatusCode::CREATED, Json(to_response(&question))))
}

pub async fn add_ich_oder_du(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<AddIchOderDuRequest>,
) -> AppResult<(StatusCode, Json<QuestionResponse>)> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let code_vo =
        GameCode::from_string(code).map_err(|e| AppError::BadRequest { message: e })?;
    let session = session_repo
        .find_by_code(&code_vo)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound {
            resource: "Session".into(),
        })?;

    let couple_answer = CoupleAnswer::from_str(&req.correct_answer)
        .map_err(|e| AppError::BadRequest { message: e.to_string() })?;

    let question = Question::create_ich_oder_du(
        session.id,
        req.text,
        couple_answer,
        req.order_index.unwrap_or(0),
    );

    let question_repo = SeaOrmQuestionRepository::new(state.db.connection().clone());
    question_repo
        .save(&question)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    Ok((StatusCode::CREATED, Json(to_response(&question))))
}

pub async fn list_questions(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
) -> AppResult<Json<Vec<QuestionResponse>>> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let code_vo =
        GameCode::from_string(code).map_err(|e| AppError::BadRequest { message: e })?;
    let session = session_repo
        .find_by_code(&code_vo)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound {
            resource: "Session".into(),
        })?;

    let question_repo = SeaOrmQuestionRepository::new(state.db.connection().clone());
    let questions = question_repo
        .find_by_session_id(session.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    Ok(Json(questions.iter().map(to_response).collect()))
}

pub async fn delete_question(
    Extension(state): Extension<AppState>,
    Path((_code, question_id)): Path<(String, uuid::Uuid)>,
) -> AppResult<StatusCode> {
    let question_repo = SeaOrmQuestionRepository::new(state.db.connection().clone());
    question_repo
        .delete(question_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_score_config(
    Extension(state): Extension<AppState>,
    Path(code): Path<String>,
    Json(req): Json<UpdateScoreConfigRequest>,
) -> AppResult<Json<ScoreConfigResponse>> {
    let session_repo = SeaOrmGameSessionRepository::new(state.db.connection().clone());
    let code_vo =
        GameCode::from_string(code).map_err(|e| AppError::BadRequest { message: e })?;
    let session = session_repo
        .find_by_code(&code_vo)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .ok_or_else(|| AppError::NotFound {
            resource: "Session".into(),
        })?;

    let config_repo = SeaOrmScoreConfigRepository::new(state.db.connection().clone());
    let mut config = config_repo
        .find_by_session(session.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?
        .unwrap_or_else(|| ScoreConfig::default_for_session(session.id));

    if let Some(v) = req.tier1_max_seconds {
        config.tier1_max_seconds = v;
    }
    if let Some(v) = req.tier2_max_seconds {
        config.tier2_max_seconds = v;
    }
    if let Some(v) = req.tier1_multiplier {
        config.tier1_multiplier = v;
    }
    if let Some(v) = req.tier2_multiplier {
        config.tier2_multiplier = v;
    }
    if let Some(v) = req.tier3_multiplier {
        config.tier3_multiplier = v;
    }
    if let Some(v) = req.perfect_match_multiplier {
        config.perfect_match_multiplier = v;
    }
    if let Some(v) = req.catchup_multiplier {
        config.catchup_multiplier = v;
    }
    if let Some(v) = req.catchup_threshold_percent {
        config.catchup_threshold_percent = v;
    }
    if let Some(v) = req.base_points {
        config.base_points = v;
    }

    config_repo
        .upsert(&config)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    Ok(Json(ScoreConfigResponse {
        session_id: config.session_id,
        tier1_max_seconds: config.tier1_max_seconds,
        tier2_max_seconds: config.tier2_max_seconds,
        tier1_multiplier: config.tier1_multiplier,
        tier2_multiplier: config.tier2_multiplier,
        tier3_multiplier: config.tier3_multiplier,
        perfect_match_multiplier: config.perfect_match_multiplier,
        catchup_multiplier: config.catchup_multiplier,
        catchup_threshold_percent: config.catchup_threshold_percent,
        base_points: config.base_points,
    }))
}

fn to_response(q: &Question) -> QuestionResponse {
    QuestionResponse {
        id: q.id,
        question_type: q.question_type.as_str().to_string(),
        text: q.text.clone(),
        option_a: q.option_a.clone(),
        option_b: q.option_b.clone(),
        option_c: q.option_c.clone(),
        option_d: q.option_d.clone(),
        correct_answer: q.correct_answer.clone(),
        order_index: q.order_index,
        points: q.points,
        created_at: q.created_at,
    }
}
