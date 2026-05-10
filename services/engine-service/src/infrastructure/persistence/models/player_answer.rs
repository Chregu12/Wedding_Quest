use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "player_answers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub round_id: Uuid,
    pub player_id: Uuid,
    pub player_name: String,
    pub answer: String,
    pub is_correct: bool,
    pub answered_at: DateTimeWithTimeZone,
    pub time_taken_seconds: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::game_round::Entity",
        from = "Column::RoundId",
        to = "super::game_round::Column::Id"
    )]
    GameRound,
}

impl Related<super::game_round::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameRound.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
