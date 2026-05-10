use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait,
};

use crate::domain::game::entity::{GameState, GameStatus};
use crate::errors::AppError;

use super::models::game_state::{ActiveModel, Entity};

pub struct GameStateRepository {
    db: DatabaseConnection,
}

impl GameStateRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, state: &GameState) -> Result<(), AppError> {
        let model = ActiveModel {
            session_code: Set(state.session_code.clone()),
            status: Set(state.status.as_str().to_string()),
            current_round_id: Set(state.current_round_id),
            current_round_number: Set(state.current_round_number),
            total_questions: Set(state.total_questions),
            updated_at: Set(state.updated_at.fixed_offset()),
        };
        model.save(&self.db).await?;
        Ok(())
    }

    pub async fn find(&self, session_code: &str) -> Result<Option<GameState>, AppError> {
        let model = Entity::find_by_id(session_code).one(&self.db).await?;
        Ok(model.map(|m| GameState {
            session_code: m.session_code,
            status: GameStatus::from_str(&m.status).unwrap_or(GameStatus::Waiting),
            current_round_id: m.current_round_id,
            current_round_number: m.current_round_number,
            total_questions: m.total_questions,
            updated_at: DateTime::from(m.updated_at),
        }))
    }
}
