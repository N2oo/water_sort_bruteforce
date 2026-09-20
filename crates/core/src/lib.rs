//! Pure domain of the water sort puzzle.
//!
//! This crate is the hexagon: it knows nothing about HTTP, databases or the
//! filesystem. The game rules (`pipe`, `game`) and the bruteforce solver
//! (`solver`) are the original implementation and must stay untouched; the
//! `color` and `puzzle` modules only provide a transport neutral way to carry a
//! board in and out of the domain.

pub mod color;
pub mod game;
pub mod pipe;
pub mod puzzle;
pub mod solver;

pub use color::{ColorParseError, color_name, parse_color};
pub use game::Game;
pub use pipe::{Color, Pipe};
pub use puzzle::{MAX_PIPE_CAPACITY, Move, MoveError, PipeDefinition, Puzzle, PuzzleError};
pub use solver::solve_puzzle_by_bruteforce;
