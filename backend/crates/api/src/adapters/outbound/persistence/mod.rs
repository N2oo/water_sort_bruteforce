//! Driven adapter: PostgreSQL through SeaORM.
//!
//! Boards and histories are stored as `jsonb` columns, which keeps the storage
//! schema stable whatever the shape of a board.

pub mod entities;
mod game_sessions;
mod in_memory;
mod scenarios;

use anyhow::Context;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use uuid::Uuid;
use water_sort_migration::Migrator;

pub use game_sessions::PostgresGameSessionRepository;
pub use in_memory::{InMemoryGameSessionRepository, InMemoryScenarioRepository};
pub use scenarios::PostgresScenarioRepository;

use crate::application::RepositoryError;

/// Open the connection pool used by every repository.
pub async fn connect(database_url: &str, max_connections: u32) -> anyhow::Result<DatabaseConnection> {
    let mut options = ConnectOptions::new(database_url.to_owned());
    options
        .max_connections(max_connections)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);

    Database::connect(options).await.context("cannot reach the database")
}

/// Bring the schema up to date.
pub async fn migrate(connection: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(connection, None).await.context("cannot apply the migrations")
}

fn backend(error: DbErr) -> RepositoryError {
    RepositoryError::Backend(anyhow::Error::new(error))
}

fn corrupted(id: Uuid, error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Corrupted { id, reason: error.to_string() }
}
