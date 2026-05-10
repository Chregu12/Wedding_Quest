use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::game::entity::{GameRound, RoundStatus};
use crate::errors::AppError;

use super::models::game_round::{ActiveModel, Column, Entity};

pub struct GameRoundRepository {
    db: DatabaseConnection,
}

impl GameRoundRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn save(&self, round: &GameRound) -> Result<(), AppError> {
        let model = ActiveModel {
            id: Set(round.id),
            session_code: Set(round.session_code.clone()),
            question_id: Set(round.question_id),
            question_type: Set(round.question_type.clone()),
            question_text: Set(round.question_text.clone()),
            option_a: Set(round.option_a.clone()),
            option_b: Set(round.option_b.clone()),
            option_c: Set(round.option_c.clone()),
            option_d: Set(round.option_d.clone()),
            correct_answer: Set(round.correct_answer.clone()),
            ich_oder_du_id: Set(round.ich_oder_du_id),
            ich_oder_du_text: Set(round.ich_oder_du_text.clone()),
            ich_oder_du_correct: Set(round.ich_oder_du_correct.clone()),
            couple_answer: Set(round.couple_answer.clone()),
            status: Set(round.status.as_str().to_string()),
            round_number: Set(round.round_number),
            started_at: Set(round.started_at.fixed_offset()),
            closed_at: Set(round.closed_at.map(|t| t.fixed_offset())),
        };
        Entity::insert(model).exec(&self.db).await?;
        Ok(())
    }

    pub async fn update(&self, round: &GameRound) -> Result<(), AppError> {
        let model = ActiveModel {
            id: Set(round.id),
            session_code: Set(round.session_code.clone()),
            question_id: Set(round.question_id),
            question_type: Set(round.question_type.clone()),
            question_text: Set(round.question_text.clone()),
            option_a: Set(round.option_a.clone()),
            option_b: Set(round.option_b.clone()),
            option_c: Set(round.option_c.clone()),
            option_d: Set(round.option_d.clone()),
            correct_answer: Set(round.correct_answer.clone()),
            ich_oder_du_id: Set(round.ich_oder_du_id),
            ich_oder_du_text: Set(round.ich_oder_du_text.clone()),
            ich_oder_du_correct: Set(round.ich_oder_du_correct.clone()),
            couple_answer: Set(round.couple_answer.clone()),
            status: Set(round.status.as_str().to_string()),
            round_number: Set(round.round_number),
            started_at: Set(round.started_at.fixed_offset()),
            closed_at: Set(round.closed_at.map(|t| t.fixed_offset())),
        };
        model.update(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<GameRound>, AppError> {
        let model = Entity::find_by_id(id).one(&self.db).await?;
        model.map(map_model).transpose()
    }

    pub async fn find_active_round(
        &self,
        session_code: &str,
    ) -> Result<Option<GameRound>, AppError> {
        let model = Entity::find()
            .filter(Column::SessionCode.eq(session_code))
            .filter(Column::Status.eq("active"))
            .one(&self.db)
            .await?;
        model.map(map_model).transpose()
    }

    pub async fn find_by_code_and_number(
        &self,
        session_code: &str,
        round_number: i32,
    ) -> Result<Option<GameRound>, AppError> {
        let model = Entity::find()
            .filter(Column::SessionCode.eq(session_code))
            .filter(Column::RoundNumber.eq(round_number))
            .one(&self.db)
            .await?;
        model.map(map_model).transpose()
    }
}

fn map_model(m: super::models::game_round::Model) -> Result<GameRound, AppError> {
    Ok(GameRound {
        id: m.id,
        session_code: m.session_code,
        question_id: m.question_id,
        question_type: m.question_type,
        question_text: m.question_text,
        option_a: m.option_a,
        option_b: m.option_b,
        option_c: m.option_c,
        option_d: m.option_d,
        correct_answer: m.correct_answer,
        ich_oder_du_id: m.ich_oder_du_id,
        ich_oder_du_text: m.ich_oder_du_text,
        ich_oder_du_correct: m.ich_oder_du_correct,
        couple_answer: m.couple_answer,
        status: RoundStatus::from_str(&m.status)
            .map_err(|e| AppError::BadRequest(e))?,
        round_number: m.round_number,
        started_at: DateTime::from(m.started_at),
        closed_at: m.closed_at.map(|t| DateTime::from(t)),
    })
}
