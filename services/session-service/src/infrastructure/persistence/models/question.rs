use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "questions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub question_type: String,
    pub text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub order_index: i32,
    pub points: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::game_session::Entity",
        from = "Column::SessionId",
        to = "super::game_session::Column::Id"
    )]
    GameSession,
}

impl Related<super::game_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
