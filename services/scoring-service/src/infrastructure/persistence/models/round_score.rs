use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "round_scores")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub round_id: Uuid,
    pub session_code: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub base_points: i32,
    pub time_multiplier: f64,
    pub final_points: i32,
    pub is_correct: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
