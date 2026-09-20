use chrono::{DateTime, Utc};
use uuid::Uuid;
use water_sort_core::{Move, MoveError, Puzzle};
use water_sort_format::Board;

use super::DomainError;

/// Where a game stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Running,
    Solved,
}

impl GameStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Solved => "solved",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "running" => Some(Self::Running),
            "solved" => Some(Self::Solved),
            _ => None,
        }
    }
}

/// One pour, and the board it produced.
///
/// Storing the resulting board makes a rollback a truncation: dropping the tail
/// of the list is enough to restore the previous state.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayedMove {
    pub id: Uuid,
    pub sequence: i32,
    pub from_pipe: Uuid,
    pub to_pipe: Uuid,
    pub board_after: Board,
    pub played_at: DateTime<Utc>,
}

/// A running game: a board, everything played on it so far, and where it came
/// from.
#[derive(Debug, Clone, PartialEq)]
pub struct GameSession {
    pub id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub name: Option<String>,
    pub status: GameStatus,
    pub initial_board: Board,
    pub current_board: Board,
    pub moves: Vec<PlayedMove>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GameSession {
    /// Start a game from `board`, optionally remembering the scenario it came
    /// from.
    pub fn start(
        board: Board,
        scenario_id: Option<Uuid>,
        name: Option<String>,
    ) -> Result<Self, DomainError> {
        let solved = board.to_puzzle()?.is_solved();
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            scenario_id,
            name,
            status: if solved { GameStatus::Solved } else { GameStatus::Running },
            initial_board: board.clone(),
            current_board: board,
            moves: Vec::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// The board as the rules see it.
    pub fn puzzle(&self) -> Result<Puzzle, DomainError> {
        Ok(self.current_board.to_puzzle()?)
    }

    pub fn is_solved(&self) -> bool {
        self.status == GameStatus::Solved
    }

    /// Pour `from` into `to`, appending the resulting board to the history.
    pub fn play(&mut self, from: Uuid, to: Uuid) -> Result<&PlayedMove, DomainError> {
        if self.is_solved() {
            return Err(DomainError::AlreadySolved);
        }
        if from == to {
            return Err(DomainError::SamePipe(from));
        }
        if !self.current_board.contains(from) {
            return Err(DomainError::UnknownPipe(from));
        }
        if !self.current_board.contains(to) {
            return Err(DomainError::UnknownPipe(to));
        }

        let movement = Move::new(from.to_string(), to.to_string());
        let played = self.puzzle()?.play(&movement).map_err(|error| match error {
            MoveError::IllegalMove { .. } => DomainError::IllegalMove { from, to },
            MoveError::SamePipe(_) => DomainError::SamePipe(from),
            MoveError::UnknownPipe(_) => DomainError::UnknownPipe(from),
        })?;

        let now = Utc::now();
        self.current_board = self.current_board.with_puzzle(&played);
        self.status = if played.is_solved() { GameStatus::Solved } else { GameStatus::Running };
        self.updated_at = now;
        self.moves.push(PlayedMove {
            id: Uuid::new_v4(),
            sequence: self.moves.len() as i32 + 1,
            from_pipe: from,
            to_pipe: to,
            board_after: self.current_board.clone(),
            played_at: now,
        });

        Ok(self.moves.last().expect("a move was just pushed"))
    }

    /// Undo the `steps` last moves. The rolled back moves are **dropped**: they
    /// leave the history for good, and the next move starts again from the
    /// restored board.
    pub fn roll_back(&mut self, steps: usize) -> Result<Vec<PlayedMove>, DomainError> {
        if steps == 0 {
            return Err(DomainError::NothingToRollBack);
        }
        if steps > self.moves.len() {
            return Err(DomainError::NotEnoughMoves {
                requested: steps,
                played: self.moves.len(),
            });
        }

        let dropped = self.moves.split_off(self.moves.len() - steps);
        self.current_board = match self.moves.last() {
            Some(previous) => previous.board_after.clone(),
            None => self.initial_board.clone(),
        };
        self.status = if self.current_board.to_puzzle()?.is_solved() {
            GameStatus::Solved
        } else {
            GameStatus::Running
        };
        self.updated_at = Utc::now();

        Ok(dropped)
    }

    /// Undo every move played so far.
    pub fn reset(&mut self) -> Result<Vec<PlayedMove>, DomainError> {
        if self.moves.is_empty() {
            return Err(DomainError::NothingToRollBack);
        }
        self.roll_back(self.moves.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        serde_json::from_str(
            r#"{"pipes": {"P1": ["Red", "Blue"], "P2": ["Blue", "Red"], "P3": []}}"#,
        )
        .unwrap()
    }

    fn session() -> GameSession {
        GameSession::start(board(), None, Some("test".into())).unwrap()
    }

    fn pipe(session: &GameSession, label: &str) -> Uuid {
        session.current_board.id_of(label).unwrap()
    }

    #[test]
    fn starts_running_with_an_empty_history() {
        let session = session();

        assert_eq!(session.status, GameStatus::Running);
        assert!(session.moves.is_empty());
        assert_eq!(session.current_board, session.initial_board);
    }

    #[test]
    fn playing_records_the_move_and_the_resulting_board() {
        let mut session = session();
        let (from, to) = (pipe(&session, "P1"), pipe(&session, "P3"));

        let played = session.play(from, to).unwrap().clone();

        assert_eq!(played.sequence, 1);
        assert_eq!((played.from_pipe, played.to_pipe), (from, to));
        assert_eq!(played.board_after, session.current_board);
        assert_ne!(session.current_board, session.initial_board);
        assert_eq!(session.moves.len(), 1);
    }

    #[test]
    fn refuses_moves_the_rules_reject() {
        let mut session = session();
        let (from, to) = (pipe(&session, "P1"), pipe(&session, "P2"));

        assert_eq!(session.play(from, to).unwrap_err(), DomainError::IllegalMove { from, to });
        assert_eq!(session.play(from, from).unwrap_err(), DomainError::SamePipe(from));

        let unknown = Uuid::new_v4();
        assert_eq!(session.play(unknown, to).unwrap_err(), DomainError::UnknownPipe(unknown));
        assert!(session.moves.is_empty());
    }

    #[test]
    fn rolling_back_restores_the_previous_board_and_drops_the_moves() {
        let mut session = session();
        let (p1, p2, p3) = (pipe(&session, "P1"), pipe(&session, "P2"), pipe(&session, "P3"));
        session.play(p1, p3).unwrap();
        let after_first = session.current_board.clone();
        session.play(p2, p1).unwrap();

        let dropped = session.roll_back(1).unwrap();

        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].sequence, 2);
        assert_eq!(session.moves.len(), 1);
        assert_eq!(session.current_board, after_first);
    }

    #[test]
    fn rolling_back_everything_restores_the_initial_board() {
        let mut session = session();
        let (p1, p2, p3) = (pipe(&session, "P1"), pipe(&session, "P2"), pipe(&session, "P3"));
        session.play(p1, p3).unwrap();
        session.play(p2, p1).unwrap();

        let dropped = session.reset().unwrap();

        assert_eq!(dropped.len(), 2);
        assert!(session.moves.is_empty());
        assert_eq!(session.current_board, session.initial_board);
    }

    #[test]
    fn replaying_after_a_rollback_renumbers_the_history() {
        let mut session = session();
        let (p1, p2, p3) = (pipe(&session, "P1"), pipe(&session, "P2"), pipe(&session, "P3"));
        session.play(p1, p3).unwrap();
        session.play(p2, p1).unwrap();
        session.roll_back(1).unwrap();

        let replayed = session.play(p2, p1).unwrap();

        assert_eq!(replayed.sequence, 2);
        assert_eq!(session.moves.len(), 2);
        assert_eq!(
            session.moves.iter().map(|m| m.sequence).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn refuses_impossible_rollbacks() {
        let mut session = session();

        assert_eq!(session.roll_back(0).unwrap_err(), DomainError::NothingToRollBack);
        assert_eq!(
            session.roll_back(3).unwrap_err(),
            DomainError::NotEnoughMoves { requested: 3, played: 0 }
        );
        assert_eq!(session.reset().unwrap_err(), DomainError::NothingToRollBack);
    }

    #[test]
    fn a_solved_board_closes_the_game_and_reopens_on_rollback() {
        let almost: Board = serde_json::from_str(
            r#"{"pipes": {"P1": ["Red", "Red", "Red"], "P2": ["Red"], "P3": []}}"#,
        )
        .unwrap();
        let mut session = GameSession::start(almost, None, None).unwrap();
        let (p2, p1) = (session.current_board.id_of("P2").unwrap(), session.current_board.id_of("P1").unwrap());

        session.play(p2, p1).unwrap();
        assert_eq!(session.status, GameStatus::Solved);
        assert_eq!(session.play(p1, p2).unwrap_err(), DomainError::AlreadySolved);

        session.roll_back(1).unwrap();
        assert_eq!(session.status, GameStatus::Running);
    }
}
