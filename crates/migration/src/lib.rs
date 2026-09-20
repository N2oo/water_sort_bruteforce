//! Schema of the water sort API.

pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_scenarios;
mod m20240101_000002_create_game_sessions;
mod m20240101_000003_create_game_moves;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_scenarios::Migration),
            Box::new(m20240101_000002_create_game_sessions::Migration),
            Box::new(m20240101_000003_create_game_moves::Migration),
        ]
    }
}

/// Table and column names, shared with the SeaORM entities.
#[derive(DeriveIden)]
pub enum Scenarios {
    Table,
    Id,
    Name,
    Description,
    Board,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum GameSessions {
    Table,
    Id,
    ScenarioId,
    Name,
    Status,
    InitialBoard,
    CurrentBoard,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum GameMoves {
    Table,
    Id,
    SessionId,
    Sequence,
    FromPipe,
    ToPipe,
    BoardAfter,
    PlayedAt,
}
