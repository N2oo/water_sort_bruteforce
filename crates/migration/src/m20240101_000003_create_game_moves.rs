use sea_orm_migration::prelude::*;

use crate::{GameMoves, GameSessions};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GameMoves::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GameMoves::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GameMoves::SessionId).uuid().not_null())
                    .col(ColumnDef::new(GameMoves::Sequence).integer().not_null())
                    .col(ColumnDef::new(GameMoves::FromPipe).uuid().not_null())
                    .col(ColumnDef::new(GameMoves::ToPipe).uuid().not_null())
                    .col(ColumnDef::new(GameMoves::BoardAfter).json_binary().not_null())
                    .col(
                        ColumnDef::new(GameMoves::PlayedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_game_moves_session")
                            .from(GameMoves::Table, GameMoves::SessionId)
                            .to(GameSessions::Table, GameSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // A session never holds two moves at the same position: rolling back
        // drops the tail instead of shadowing it.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_moves_session_sequence")
                    .table(GameMoves::Table)
                    .col(GameMoves::SessionId)
                    .col(GameMoves::Sequence)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GameMoves::Table).to_owned()).await
    }
}
