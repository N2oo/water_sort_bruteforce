use sea_orm_migration::prelude::*;

use crate::{GameSessions, Scenarios};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GameSessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GameSessions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GameSessions::ScenarioId).uuid().null())
                    .col(ColumnDef::new(GameSessions::Name).string().null())
                    .col(ColumnDef::new(GameSessions::Status).string().not_null())
                    .col(ColumnDef::new(GameSessions::InitialBoard).json_binary().not_null())
                    .col(ColumnDef::new(GameSessions::CurrentBoard).json_binary().not_null())
                    .col(
                        ColumnDef::new(GameSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GameSessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_game_sessions_scenario")
                            .from(GameSessions::Table, GameSessions::ScenarioId)
                            .to(Scenarios::Table, Scenarios::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_sessions_created_at")
                    .table(GameSessions::Table)
                    .col(GameSessions::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GameSessions::Table).to_owned()).await
    }
}
