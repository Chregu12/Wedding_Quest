use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "game_rounds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_code: String,
    pub question_id: Uuid,
    pub question_type: String,
    pub question_text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub ich_oder_du_id: Option<Uuid>,
    pub ich_oder_du_text: Option<String>,
    pub ich_oder_du_correct: Option<String>,
    pub couple_answer: Option<String>,
    pub status: String,
    pub round_number: i32,
    pub started_at: DateTimeWithTimeZone,
    pub closed_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::player_answer::Entity")]
    PlayerAnswer,
}

impl Related<super::player_answer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayerAnswer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
