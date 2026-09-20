//! Use cases around solving, for a board, a scenario or a running game.

use std::sync::Arc;

use uuid::Uuid;
use water_sort_format::Board;

use super::{ApplicationError, GameSessionRepository, PuzzleSolver, ScenarioRepository, Solution};

#[derive(Clone)]
pub struct SolvingService {
    solver: Arc<dyn PuzzleSolver>,
    sessions: Arc<dyn GameSessionRepository>,
    scenarios: Arc<dyn ScenarioRepository>,
}

impl SolvingService {
    pub fn new(
        solver: Arc<dyn PuzzleSolver>,
        sessions: Arc<dyn GameSessionRepository>,
        scenarios: Arc<dyn ScenarioRepository>,
    ) -> Self {
        Self { solver, sessions, scenarios }
    }

    /// Solve a board that was submitted on the spot, storing nothing.
    pub async fn solve_board(&self, board: &Board) -> Result<Solution, ApplicationError> {
        Ok(self.solver.solve(board).await?)
    }

    /// Solve a saved scenario, from its initial board.
    pub async fn solve_scenario(&self, id: Uuid) -> Result<Solution, ApplicationError> {
        let scenario = self
            .scenarios
            .find(id)
            .await?
            .ok_or(ApplicationError::NotFound { resource: "scenario", id })?;
        self.solve_board(&scenario.board).await
    }

    /// Solve a running game **from where it currently stands**, so the returned
    /// moves can be played as is.
    pub async fn solve_session(&self, id: Uuid) -> Result<Solution, ApplicationError> {
        let session = self
            .sessions
            .find(id)
            .await?
            .ok_or(ApplicationError::NotFound { resource: "game", id })?;
        self.solve_board(&session.current_board).await
    }
}
