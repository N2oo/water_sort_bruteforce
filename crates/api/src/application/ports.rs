//! The ports of the hexagon: what the use cases need from the outside world.

use async_trait::async_trait;
use uuid::Uuid;
use water_sort_format::Board;

use crate::domain::{GameSession, Scenario};

/// A slice of a listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub limit: u64,
    pub offset: u64,
}

impl Page {
    pub const DEFAULT_LIMIT: u64 = 50;
    pub const MAX_LIMIT: u64 = 200;

    pub fn new(limit: Option<u64>, offset: Option<u64>) -> Self {
        Self {
            limit: limit.unwrap_or(Self::DEFAULT_LIMIT).clamp(1, Self::MAX_LIMIT),
            offset: offset.unwrap_or(0),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// Anything that goes wrong while talking to the storage.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("storage failure: {0}")]
    Backend(#[from] anyhow::Error),

    #[error("stored row '{id}' cannot be read back: {reason}")]
    Corrupted { id: Uuid, reason: String },
}

/// Driven port: persistence of saved scenarios.
#[async_trait]
pub trait ScenarioRepository: Send + Sync {
    async fn save(&self, scenario: &Scenario) -> Result<(), RepositoryError>;
    async fn find(&self, id: Uuid) -> Result<Option<Scenario>, RepositoryError>;
    async fn list(&self, page: Page) -> Result<Vec<Scenario>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
}

/// Driven port: persistence of game sessions, history included.
///
/// `save` stores the whole aggregate, which is what makes a rollback durable:
/// the moves that left the aggregate must leave the storage too.
#[async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn save(&self, session: &GameSession) -> Result<(), RepositoryError>;
    async fn find(&self, id: Uuid) -> Result<Option<GameSession>, RepositoryError>;
    async fn list(&self, page: Page) -> Result<Vec<GameSession>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
}

/// A move of a solution, with the labels that make it readable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionMove {
    pub from_pipe: Uuid,
    pub to_pipe: Uuid,
    pub from_label: String,
    pub to_label: String,
}

/// The answer of the solver for a given board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    pub solved: bool,
    pub moves: Vec<SolutionMove>,
}

impl Solution {
    pub fn unsolvable() -> Self {
        Self { solved: false, moves: Vec::new() }
    }
}

/// What the solver can fail with.
#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("the board cannot be solved as submitted: {0}")]
    InvalidBoard(String),

    #[error("no solution found within {seconds}s")]
    TimedOut { seconds: u64 },

    #[error("the solver stopped unexpectedly: {0}")]
    Crashed(String),
}

/// Driven port: the solving algorithm itself.
#[async_trait]
pub trait PuzzleSolver: Send + Sync {
    async fn solve(&self, board: &Board) -> Result<Solution, SolverError>;
}
