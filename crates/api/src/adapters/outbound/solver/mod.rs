//! Driven adapter: the historical bruteforce solver.
//!
//! The algorithm is the untouched `water_sort_core::solve_puzzle_by_bruteforce`.
//! This adapter only makes it usable from an async request handler: the search
//! is deeply recursive, so it runs on a dedicated thread with a large stack, and
//! a request never hangs forever thanks to a configurable deadline.

use std::time::Duration;

use async_trait::async_trait;
use water_sort_core::{Move, solve_puzzle_by_bruteforce};
use water_sort_format::Board;

use crate::application::{PuzzleSolver, Solution, SolutionMove, SolverError};

/// Stack given to the solver thread; the recursion goes up to 1000 levels deep.
pub const DEFAULT_STACK_SIZE: usize = 256 * 1024 * 1024;

pub struct BruteforceSolver {
    timeout: Duration,
    stack_size: usize,
}

impl BruteforceSolver {
    pub fn new(timeout: Duration, stack_size: usize) -> Self {
        Self { timeout, stack_size }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self::new(timeout, DEFAULT_STACK_SIZE)
    }
}

#[async_trait]
impl PuzzleSolver for BruteforceSolver {
    async fn solve(&self, board: &Board) -> Result<Solution, SolverError> {
        let puzzle = board
            .to_puzzle()
            .map_err(|error| SolverError::InvalidBoard(error.to_string()))?;

        let (sender, receiver) = tokio::sync::oneshot::channel();
        std::thread::Builder::new()
            .name("water-sort-solver".into())
            .stack_size(self.stack_size)
            .spawn(move || {
                let _ = sender.send(solve_puzzle_by_bruteforce(&puzzle.to_game()));
            })
            .map_err(|error| SolverError::Crashed(error.to_string()))?;

        // On a timeout the thread is left running: the bruteforce search has no
        // cancellation point. It finishes on its own and drops its result.
        match tokio::time::timeout(self.timeout, receiver).await {
            Ok(Ok(Some(moves))) => Ok(Solution { solved: true, moves: translate(board, &moves) }),
            Ok(Ok(None)) => Ok(Solution::unsolvable()),
            Ok(Err(_)) => Err(SolverError::Crashed("the solver thread stopped".into())),
            Err(_) => Err(SolverError::TimedOut { seconds: self.timeout.as_secs() }),
        }
    }
}

/// Turn the solver's `"uuid->uuid"` moves into moves the API can hand out.
fn translate(board: &Board, moves: &[String]) -> Vec<SolutionMove> {
    moves
        .iter()
        .filter_map(|raw| Move::parse(raw))
        .filter_map(|movement| {
            let from_pipe = movement.from.parse().ok()?;
            let to_pipe = movement.to.parse().ok()?;
            Some(SolutionMove {
                from_pipe,
                to_pipe,
                from_label: board.label_of(from_pipe)?.to_string(),
                to_label: board.label_of(to_pipe)?.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(raw: &str) -> Board {
        serde_json::from_str(raw).unwrap()
    }

    #[tokio::test]
    async fn solves_a_board_and_names_the_pipes() {
        let board = board(
            r#"{"pipes": {"P1": ["Red", "Red", "Red", "Blue"], "P2": ["Blue", "Blue", "Blue", "Red"], "P3": []}}"#,
        );

        let solution = BruteforceSolver::with_timeout(Duration::from_secs(30))
            .solve(&board)
            .await
            .unwrap();

        assert!(solution.solved);
        assert!(!solution.moves.is_empty());
        assert!(
            solution
                .moves
                .iter()
                .all(|movement| ["P1", "P2", "P3"].contains(&movement.from_label.as_str()))
        );

        // the returned moves really do solve the board
        let moves: Vec<Move> = solution
            .moves
            .iter()
            .map(|m| Move::new(m.from_pipe.to_string(), m.to_pipe.to_string()))
            .collect();
        assert!(board.to_puzzle().unwrap().play_all(&moves).unwrap().is_solved());
    }

    #[tokio::test]
    async fn reports_an_unsolvable_board() {
        let board = board(r#"{"pipes": {"P1": ["Red", "Blue"], "P2": ["Blue", "Red"]}}"#);

        let solution = BruteforceSolver::with_timeout(Duration::from_secs(30))
            .solve(&board)
            .await
            .unwrap();

        assert!(!solution.solved);
        assert!(solution.moves.is_empty());
    }

    #[tokio::test]
    async fn an_already_solved_board_needs_no_move() {
        let board = board(r#"{"pipes": {"P1": ["Red", "Red", "Red", "Red"], "P2": []}}"#);

        let solution = BruteforceSolver::with_timeout(Duration::from_secs(30))
            .solve(&board)
            .await
            .unwrap();

        assert!(solution.solved);
        assert!(solution.moves.is_empty());
    }
}
