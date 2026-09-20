//! Use cases, and the ports they need.
//!
//! Nothing here knows whether a board arrives over HTTP or whether it is stored
//! in Postgres: the services only talk to the traits declared in [`ports`].

pub mod ports;
mod scenarios;
mod sessions;
mod solving;

use uuid::Uuid;

pub use ports::{
    GameSessionRepository, Page, PuzzleSolver, RepositoryError, Solution, SolutionMove,
    ScenarioRepository, SolverError,
};
pub use scenarios::ScenarioService;
pub use sessions::{GameService, StartGame};
pub use solving::SolvingService;

use crate::domain::DomainError;

/// What a use case can fail with.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("no {resource} with id '{id}'")]
    NotFound { resource: &'static str, id: Uuid },

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Solver(#[from] SolverError),

    #[error("{0}")]
    Invalid(String),
}
