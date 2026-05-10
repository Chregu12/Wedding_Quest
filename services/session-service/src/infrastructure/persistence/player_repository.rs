use crate::domain::player::{
    entity::Player,
    repository::PlayerRepository,
    value_objects::PlayerName,
};
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use uuid::Uuid;

use super::models::player::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmPlayerRepository {
    db: DatabaseConnection,
}

impl SeaOrmPlayerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PlayerRepository for SeaOrmPlayerRepository {
    async fn save(&self, player: &Player) -> Result<(), AppError> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            id: Set(player.id),
            session_id: Set(player.session_id),
            display_name: Set(player.display_name.value().to_string()),
            avatar: Set(player.avatar.clone()),
            total_score: Set(player.total_score),
            is_connected: Set(player.is_connected),
            joined_at: Set(now),
            updated_at: Set(now),
        };
        Entity::insert(model).exec(&self.db).await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>, AppError> {
        match Entity::find_by_id(id).one(&self.db).await? {
            Some(m) => {
                let name = PlayerName::new(m.display_name)
                    .map_err(|e| AppError::BadRequest(e))?;
                Ok(Some(Player {
                    id: m.id,
                    session_id: m.session_id,
                    display_name: name,
                    avatar: m.avatar,
                    total_score: m.total_score,
                    is_connected: m.is_connected,
                    joined_at: DateTime::from(m.joined_at),
                    updated_at: DateTime::from(m.updated_at),
                    pending_events: vec![],
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_session(&self, session_id: Uuid) -> Result<Vec<Player>, AppError> {
        let models = Entity::find()
            .filter(Column::SessionId.eq(session_id))
            .all(&self.db)
            .await?;

        let mut players = Vec::with_capacity(models.len());
        for m in models {
            let name = PlayerName::new(m.display_name)
                .map_err(|e| AppError::BadRequest(e))?;
            players.push(Player {
                id: m.id,
                session_id: m.session_id,
                display_name: name,
                avatar: m.avatar,
                total_score: m.total_score,
                is_connected: m.is_connected,
                joined_at: DateTime::from(m.joined_at),
                updated_at: DateTime::from(m.updated_at),
                pending_events: vec![],
            });
        }
        Ok(players)
    }

    async fn exists_in_session(&self, session_id: Uuid, name: &str) -> Result<bool, AppError> {
        let count = Entity::find()
            .filter(Column::SessionId.eq(session_id))
            .filter(Column::DisplayName.eq(name))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn update(&self, player: &Player) -> Result<(), AppError> {
        let model = ActiveModel {
            id: Set(player.id),
            session_id: Set(player.session_id),
            display_name: Set(player.display_name.value().to_string()),
            avatar: Set(player.avatar.clone()),
            total_score: Set(player.total_score),
            is_connected: Set(player.is_connected),
            joined_at: Set(player.joined_at.fixed_offset()),
            updated_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.update(&self.db).await?;
        Ok(())
    }
}
