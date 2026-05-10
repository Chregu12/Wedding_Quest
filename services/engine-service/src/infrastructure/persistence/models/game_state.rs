use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "game_state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_code: String,
    pub status: String,
    pub current_round_id: Option<Uuid>,
    pub current_round_number: i32,
    pub total_questions: i32,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
