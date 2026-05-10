use crate::domain::game_session::{
    entity::GameSession,
    repository::GameSessionRepository,
    value_objects::{GameCode, SessionStatus},
};
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

use super::models::game_session::{self as gs_model, ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmGameSessionRepository {
    db: DatabaseConnection,
}

impl SeaOrmGameSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn model_to_entity(model: gs_model::Model) -> Result<GameSession, AppError> {
        Ok(GameSession {
            id: model.id,
            game_id: None,
            code: GameCode::from_string(model.code)
                .map_err(|e| AppError::BadRequest(e))?,
            status: SessionStatus::try_from(model.status)
                .map_err(|e| AppError::BadRequest(e))?,
            host_name: model.host_name,
            person_a_name: model.person_a_name,
            person_b_name: model.person_b_name,
            current_round: model.current_round,
            started_at: model.started_at.map(|t| DateTime::<Utc>::from(t)),
            ended_at: model.ended_at.map(|t| DateTime::<Utc>::from(t)),
            created_at: DateTime::<Utc>::from(model.created_at),
            updated_at: DateTime::<Utc>::from(model.updated_at),
            pending_events: vec![],
        })
    }
}

#[async_trait]
impl GameSessionRepository for SeaOrmGameSessionRepository {
    async fn save(&self, session: &GameSession) -> Result<(), AppError> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            id: Set(session.id),
            code: Set(session.code.value().to_string()),
            status: Set(session.status.as_str().to_string()),
            host_name: Set(session.host_name.clone()),
            person_a_name: Set(session.person_a_name.clone()),
            person_b_name: Set(session.person_b_name.clone()),
            current_round: Set(session.current_round),
            started_at: Set(session.started_at.map(|t| t.fixed_offset())),
            ended_at: Set(session.ended_at.map(|t| t.fixed_offset())),
            created_at: Set(now),
            updated_at: Set(now),
        };
        Entity::insert(model).exec(&self.db).await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<GameSession>, AppError> {
        match Entity::find_by_id(id).one(&self.db).await? {
            Some(model) => Ok(Some(Self::model_to_entity(model)?)),
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, code: &GameCode) -> Result<Option<GameSession>, AppError> {
        match Entity::find()
            .filter(Column::Code.eq(code.value()))
            .one(&self.db)
            .await?
        {
            Some(model) => Ok(Some(Self::model_to_entity(model)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, session: &GameSession) -> Result<(), AppError> {
        let model = ActiveModel {
            id: Set(session.id),
            code: Set(session.code.value().to_string()),
            status: Set(session.status.as_str().to_string()),
            host_name: Set(session.host_name.clone()),
            person_a_name: Set(session.person_a_name.clone()),
            person_b_name: Set(session.person_b_name.clone()),
            current_round: Set(session.current_round),
            started_at: Set(session.started_at.map(|t| t.fixed_offset())),
            ended_at: Set(session.ended_at.map(|t| t.fixed_offset())),
            created_at: Set(session.created_at.fixed_offset()),
            updated_at: Set(Utc::now().fixed_offset()),
        };
        model.update(&self.db).await?;
        Ok(())
    }
}
