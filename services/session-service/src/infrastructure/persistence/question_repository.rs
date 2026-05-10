use async_trait::async_trait;
use chrono::DateTime;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::question::{
    entity::Question,
    repository::QuestionRepository,
    value_objects::QuestionType,
};
use crate::errors::AppError;

use super::models::question::{ActiveModel, Column, Entity};

pub struct SeaOrmQuestionRepository {
    db: DatabaseConnection,
}

impl SeaOrmQuestionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl QuestionRepository for SeaOrmQuestionRepository {
    async fn save(&self, q: &Question) -> Result<(), AppError> {
        let model = ActiveModel {
            id: Set(q.id),
            session_id: Set(q.session_id),
            question_type: Set(q.question_type.as_str().to_string()),
            text: Set(q.text.clone()),
            option_a: Set(q.option_a.clone()),
            option_b: Set(q.option_b.clone()),
            option_c: Set(q.option_c.clone()),
            option_d: Set(q.option_d.clone()),
            correct_answer: Set(q.correct_answer.clone()),
            order_index: Set(q.order_index),
            points: Set(q.points),
            created_at: Set(q.created_at.fixed_offset()),
        };
        Entity::insert(model).exec(&self.db).await?;
        Ok(())
    }

    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Vec<Question>, AppError> {
        let models = Entity::find()
            .filter(Column::SessionId.eq(session_id))
            .all(&self.db)
            .await?;
        models.into_iter().map(map_model).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Question>, AppError> {
        let model = Entity::find_by_id(id).one(&self.db).await?;
        model.map(map_model).transpose()
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}

fn map_model(m: super::models::question::Model) -> Result<Question, AppError> {
    Ok(Question {
        id: m.id,
        session_id: m.session_id,
        question_type: QuestionType::from_str(&m.question_type)?,
        text: m.text,
        option_a: m.option_a,
        option_b: m.option_b,
        option_c: m.option_c,
        option_d: m.option_d,
        correct_answer: m.correct_answer,
        order_index: m.order_index,
        points: m.points,
        created_at: DateTime::from(m.created_at),
    })
}
