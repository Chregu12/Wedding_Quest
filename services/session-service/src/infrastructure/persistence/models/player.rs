use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "players")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub display_name: String,
    pub avatar: Option<String>,
    pub total_score: i64,
    pub is_connected: bool,
    pub joined_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
