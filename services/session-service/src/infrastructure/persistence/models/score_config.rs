use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "score_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: Uuid,
    pub tier1_max_seconds: i32,
    pub tier2_max_seconds: i32,
    pub tier1_multiplier: f64,
    pub tier2_multiplier: f64,
    pub tier3_multiplier: f64,
    pub perfect_match_multiplier: f64,
    pub catchup_multiplier: f64,
    pub catchup_threshold_percent: i32,
    pub base_points: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
