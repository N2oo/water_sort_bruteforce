//! Water sort HTTP API, built as a hexagon.
//!
//! ```text
//!            driving adapters                 ports                driven adapters
//!   HTTP  ──▶ adapters::inbound::http ──▶ application::* ──▶ adapters::outbound::persistence
//!                                                       └──▶ adapters::outbound::solver
//! ```
//!
//! * [`domain`] owns the lifecycle of a game session and delegates every rule to
//!   `water_sort_core`, which is shared with the CLI.
//! * [`application`] declares the ports and orchestrates the use cases.
//! * [`adapters`] plug HTTP, PostgreSQL and the bruteforce solver onto them.

pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;

use std::sync::Arc;

use adapters::inbound::http::AppState;
use adapters::outbound::persistence::{
    InMemoryGameSessionRepository, InMemoryScenarioRepository, PostgresGameSessionRepository,
    PostgresScenarioRepository, connect, migrate,
};
use adapters::outbound::solver::BruteforceSolver;
use application::{GameSessionRepository, ScenarioRepository};
use config::{Config, Storage};

/// Build the application state described by `config`, opening the database and
/// running the migrations when needed.
pub async fn build_state(config: &Config) -> anyhow::Result<AppState> {
    let (scenarios, sessions): (Arc<dyn ScenarioRepository>, Arc<dyn GameSessionRepository>) =
        match &config.storage {
            Storage::Postgres { url, max_connections, run_migrations } => {
                let connection = connect(url, *max_connections).await?;
                if *run_migrations {
                    migrate(&connection).await?;
                }
                (
                    Arc::new(PostgresScenarioRepository::new(connection.clone())),
                    Arc::new(PostgresGameSessionRepository::new(connection)),
                )
            }
            Storage::Memory => {
                tracing::warn!("STORAGE=memory: everything is lost when the process stops");
                (
                    Arc::new(InMemoryScenarioRepository::default()),
                    Arc::new(InMemoryGameSessionRepository::default()),
                )
            }
        };

    let solver = Arc::new(BruteforceSolver::new(
        config.solver_timeout,
        config.solver_stack_size,
    ));

    Ok(AppState::new(scenarios, sessions, solver))
}
