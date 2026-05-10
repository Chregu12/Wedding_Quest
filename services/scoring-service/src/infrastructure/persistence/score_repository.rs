use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::domain::scoring::entity::{PlayerScore, RoundScore};
use crate::errors::AppError;

use super::models::{
    player_score::{
        ActiveModel as PlayerActiveModel, Column as PlayerColumn, Entity as PlayerEntity,
        Model as PlayerModel,
    },
    round_score::{ActiveModel as RoundActiveModel, Entity as RoundEntity},
};

fn model_to_entity(m: PlayerModel) -> PlayerScore {
    PlayerScore {
        id: m.id,
        session_code: m.session_code,
        player_id: m.player_id,
        player_name: m.player_name,
        total_score: m.total_score,
        rounds_played: m.rounds_played,
        last_round_score: m.last_round_score,
        updated_at: m.updated_at.into(),
        lucky_boost_multiplier: m.lucky_boost_multiplier,
    }
}

pub struct ScoreRepository {
    db: DatabaseConnection,
}

impl ScoreRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Upsert aggregate player score.  Conflict target is `(session_code, player_id)`.
    pub async fn upsert_player_score(&self, score: &PlayerScore) -> Result<(), AppError> {
        let model = PlayerActiveModel {
            id: Set(score.id),
            session_code: Set(score.session_code.clone()),
            player_id: Set(score.player_id),
            player_name: Set(score.player_name.clone()),
            total_score: Set(score.total_score),
            rounds_played: Set(score.rounds_played),
            last_round_score: Set(score.last_round_score),
            updated_at: Set(score.updated_at.into()),
            lucky_boost_multiplier: Set(score.lucky_boost_multiplier),
        };

        PlayerEntity::insert(model)
            .on_conflict(
                OnConflict::columns([PlayerColumn::SessionCode, PlayerColumn::PlayerId])
                    .update_columns([
                        PlayerColumn::TotalScore,
                        PlayerColumn::RoundsPlayed,
                        PlayerColumn::LastRoundScore,
                        PlayerColumn::UpdatedAt,
                        PlayerColumn::LuckyBoostMultiplier,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

        Ok(())
    }

    /// Set a Lucky Boost multiplier for a specific player (for their next correct answer).
    pub async fn set_lucky_boost(
        &self,
        session_code: &str,
        player_id: Uuid,
        multiplier: f64,
    ) -> Result<(), AppError> {
        let model = PlayerEntity::find()
            .filter(PlayerColumn::SessionCode.eq(session_code))
            .filter(PlayerColumn::PlayerId.eq(player_id))
            .one(&self.db)
            .await?;

        if let Some(m) = model {
            let mut active: PlayerActiveModel = m.into();
            active.lucky_boost_multiplier = Set(multiplier);
            active
                .update(&self.db)
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
        }
        Ok(())
    }

    /// Insert a per-round score record (append-only).
    pub async fn insert_round_score(&self, score: &RoundScore) -> Result<(), AppError> {
        let now: chrono::DateTime<chrono::FixedOffset> =
            chrono::Utc::now().into();

        let model = RoundActiveModel {
            id: Set(score.id),
            round_id: Set(score.round_id),
            session_code: Set(score.session_code.clone()),
            player_id: Set(score.player_id),
            player_name: Set(score.player_name.clone()),
            base_points: Set(score.base_points),
            time_multiplier: Set(score.time_multiplier),
            final_points: Set(score.final_points),
            is_correct: Set(score.is_correct),
            created_at: Set(now),
        };

        RoundEntity::insert(model)
            .exec(&self.db)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

        Ok(())
    }

    /// Return all player aggregate scores for a session, ordered by total_score DESC.
    pub async fn find_by_session(&self, session_code: &str) -> Result<Vec<PlayerScore>, AppError> {
        let models = PlayerEntity::find()
            .filter(PlayerColumn::SessionCode.eq(session_code))
            .order_by_desc(PlayerColumn::TotalScore)
            .all(&self.db)
            .await?;

        Ok(models
            .into_iter()
            .map(model_to_entity)
            .collect())
    }

    /// Find a single player's aggregate score.
    pub async fn find_player(
        &self,
        session_code: &str,
        player_id: Uuid,
    ) -> Result<Option<PlayerScore>, AppError> {
        let model = PlayerEntity::find()
            .filter(PlayerColumn::SessionCode.eq(session_code))
            .filter(PlayerColumn::PlayerId.eq(player_id))
            .one(&self.db)
            .await?;

        Ok(model.map(model_to_entity))
    }
}
