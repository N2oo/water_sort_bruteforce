use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scenarios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub board: Json,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::game_session::Entity")]
    GameSession,
}

impl Related<super::game_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
