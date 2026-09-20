//! The application side of the hexagon.
//!
//! These types own the lifecycle of a *game session*: what was started from,
//! what has been played, and what the board looks like right now. They never
//! re-implement a game rule — every board transition goes through
//! [`water_sort_core::Puzzle`], which delegates to the untouched `Pipe`/`Game`
//! rules.

mod error;
mod scenario;
mod session;

pub use error::DomainError;
pub use scenario::Scenario;
pub use session::{GameSession, GameStatus, PlayedMove};
