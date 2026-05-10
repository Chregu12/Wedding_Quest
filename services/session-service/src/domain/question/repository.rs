use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;

use super::entity::Question;

#[async_trait]
pub trait QuestionRepository: Send + Sync {
    async fn save(&self, question: &Question) -> Result<(), AppError>;
    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Vec<Question>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Question>, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}
