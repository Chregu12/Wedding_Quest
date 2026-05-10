use sea_orm::{
    sea_query::OnConflict, ActiveValue::Set, DatabaseConnection, EntityTrait,
};
use uuid::Uuid;

use crate::domain::score_config::entity::ScoreConfig;
use crate::errors::AppError;

use super::models::score_config::{ActiveModel, Column, Entity};

pub struct SeaOrmScoreConfigRepository {
    db: DatabaseConnection,
}

impl SeaOrmScoreConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, config: &ScoreConfig) -> Result<(), AppError> {
        let model = ActiveModel {
            session_id: Set(config.session_id),
            tier1_max_seconds: Set(config.tier1_max_seconds),
            tier2_max_seconds: Set(config.tier2_max_seconds),
            tier1_multiplier: Set(config.tier1_multiplier),
            tier2_multiplier: Set(config.tier2_multiplier),
            tier3_multiplier: Set(config.tier3_multiplier),
            perfect_match_multiplier: Set(config.perfect_match_multiplier),
            catchup_multiplier: Set(config.catchup_multiplier),
            catchup_threshold_percent: Set(config.catchup_threshold_percent),
            base_points: Set(config.base_points),
        };
        Entity::insert(model)
            .on_conflict(
                OnConflict::column(Column::SessionId)
                    .update_columns([
                        Column::Tier1MaxSeconds,
                        Column::Tier2MaxSeconds,
                        Column::Tier1Multiplier,
                        Column::Tier2Multiplier,
                        Column::Tier3Multiplier,
                        Column::PerfectMatchMultiplier,
                        Column::CatchupMultiplier,
                        Column::CatchupThresholdPercent,
                        Column::BasePoints,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;
        Ok(())
    }

    pub async fn find_by_session(&self, session_id: Uuid) -> Result<Option<ScoreConfig>, AppError> {
        let model = Entity::find_by_id(session_id).one(&self.db).await?;
        Ok(model.map(|m| ScoreConfig {
            session_id: m.session_id,
            tier1_max_seconds: m.tier1_max_seconds,
            tier2_max_seconds: m.tier2_max_seconds,
            tier1_multiplier: m.tier1_multiplier,
            tier2_multiplier: m.tier2_multiplier,
            tier3_multiplier: m.tier3_multiplier,
            perfect_match_multiplier: m.perfect_match_multiplier,
            catchup_multiplier: m.catchup_multiplier,
            catchup_threshold_percent: m.catchup_threshold_percent,
            base_points: m.base_points,
        }))
    }
}
