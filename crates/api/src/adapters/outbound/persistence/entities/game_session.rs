use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "game_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub name: Option<String>,
    pub status: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub initial_board: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub current_board: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scenario::Entity",
        from = "Column::ScenarioId",
        to = "super::scenario::Column::Id",
        on_delete = "SetNull"
    )]
    Scenario,
    #[sea_orm(has_many = "super::game_move::Entity")]
    GameMove,
}

impl Related<super::scenario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scenario.def()
    }
}

impl Related<super::game_move::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameMove.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
