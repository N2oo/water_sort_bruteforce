use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_moves")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence: i32,
    pub from_pipe: Uuid,
    pub to_pipe: Uuid,
    #[sea_orm(column_type = "JsonBinary")]
    pub board_after: Json,
    pub played_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::game_session::Entity",
        from = "Column::SessionId",
        to = "super::game_session::Column::Id",
        on_delete = "Cascade"
    )]
    GameSession,
}

impl Related<super::game_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
