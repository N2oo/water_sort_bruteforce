use uuid::Uuid;

use water_sort_format::BoardError;

/// Everything the domain can refuse.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("pipe '{0}' does not belong to this game")]
    UnknownPipe(Uuid),

    #[error("cannot pour pipe '{0}' into itself")]
    SamePipe(Uuid),

    #[error("pouring '{from}' into '{to}' is not allowed by the rules")]
    IllegalMove { from: Uuid, to: Uuid },

    #[error("this game is already solved, no move can be played")]
    AlreadySolved,

    #[error("cannot roll back {requested} move(s), only {played} have been played")]
    NotEnoughMoves { requested: usize, played: usize },

    #[error("rolling back needs at least one step")]
    NothingToRollBack,

    #[error("the stored board is corrupted: {0}")]
    CorruptedBoard(String),
}

impl From<BoardError> for DomainError {
    fn from(error: BoardError) -> Self {
        Self::CorruptedBoard(error.to_string())
    }
}
