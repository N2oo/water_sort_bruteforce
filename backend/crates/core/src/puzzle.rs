//! Transport neutral description of a board.
//!
//! [`Game`] stores its pipes in a `HashMap` and [`Pipe`] keeps its content
//! private, which is everything the rules need but not enough to ship a board
//! over HTTP or store it in a database. A [`Puzzle`] is an *ordered* snapshot of
//! the very same data: it can be turned into a [`Game`] and back, and every
//! decision it takes (is this pour legal? how much liquid flows?) is delegated
//! to [`Pipe`] so the rules live in exactly one place.

use std::collections::HashMap;
use std::fmt;

use crate::color::color_name;
use crate::game::Game;
use crate::pipe::{Color, Pipe};

/// Number of units a pipe holds, mirroring `Pipe::is_filled`.
pub const MAX_PIPE_CAPACITY: usize = 4;

/// A single pipe: its identifier and its content, bottom first.
#[derive(Debug, Clone, PartialEq)]
pub struct PipeDefinition {
    pub identifier: String,
    pub colors: Vec<Color>,
}

impl PipeDefinition {
    pub fn new(identifier: impl Into<String>, colors: Vec<Color>) -> Self {
        Self { identifier: identifier.into(), colors }
    }

    fn to_pipe(&self) -> Pipe {
        let mut pipe = Pipe::new(self.identifier.clone());
        for color in &self.colors {
            pipe.add_color(color.clone());
        }
        pipe
    }
}

/// Why a board is not a valid puzzle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleError {
    /// Fewer than two pipes: no move could ever be played.
    NotEnoughPipes { found: usize },
    /// Two pipes share an identifier, which the rules use to tell them apart.
    DuplicateIdentifier(String),
    /// An identifier is blank.
    EmptyIdentifier,
    /// A pipe holds more than [`MAX_PIPE_CAPACITY`] units.
    PipeOverflow { identifier: String, found: usize },
}

impl fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughPipes { found } => {
                write!(f, "a puzzle needs at least 2 pipes, got {found}")
            }
            Self::DuplicateIdentifier(id) => write!(f, "duplicated pipe identifier '{id}'"),
            Self::EmptyIdentifier => write!(f, "pipe identifiers cannot be empty"),
            Self::PipeOverflow { identifier, found } => write!(
                f,
                "pipe '{identifier}' holds {found} units, the maximum is {MAX_PIPE_CAPACITY}"
            ),
        }
    }
}

impl std::error::Error for PuzzleError {}

/// Why a pour was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    UnknownPipe(String),
    SamePipe(String),
    /// The rules (`Pipe::can_pour`) reject this pour on the current board.
    IllegalMove { from: String, to: String },
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPipe(id) => write!(f, "unknown pipe '{id}'"),
            Self::SamePipe(id) => write!(f, "cannot pour pipe '{id}' into itself"),
            Self::IllegalMove { from, to } => {
                write!(f, "pouring '{from}' into '{to}' is not allowed by the rules")
            }
        }
    }
}

impl std::error::Error for MoveError {}

/// A pour, from one pipe into another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: String,
    pub to: String,
}

/// Separator used by the solver to render a move.
const ARROW: &str = "->";

impl Move {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self { from: from.into(), to: to.into() }
    }

    /// Read back a move rendered by the solver, e.g. `"P1->P13"`.
    pub fn parse(raw: &str) -> Option<Self> {
        let (from, to) = raw.split_once(ARROW)?;
        if from.is_empty() || to.is_empty() {
            return None;
        }
        Some(Self::new(from, to))
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.from, ARROW, self.to)
    }
}

/// An ordered, validated board.
#[derive(Debug, Clone, PartialEq)]
pub struct Puzzle {
    pipes: Vec<PipeDefinition>,
}

impl Puzzle {
    /// Validate a board. The order of `pipes` is preserved everywhere.
    pub fn new(pipes: Vec<PipeDefinition>) -> Result<Self, PuzzleError> {
        if pipes.len() < 2 {
            return Err(PuzzleError::NotEnoughPipes { found: pipes.len() });
        }

        let mut seen: Vec<&str> = Vec::with_capacity(pipes.len());
        for pipe in &pipes {
            if pipe.identifier.trim().is_empty() {
                return Err(PuzzleError::EmptyIdentifier);
            }
            if seen.contains(&pipe.identifier.as_str()) {
                return Err(PuzzleError::DuplicateIdentifier(pipe.identifier.clone()));
            }
            if pipe.colors.len() > MAX_PIPE_CAPACITY {
                return Err(PuzzleError::PipeOverflow {
                    identifier: pipe.identifier.clone(),
                    found: pipe.colors.len(),
                });
            }
            seen.push(&pipe.identifier);
        }

        Ok(Self { pipes })
    }

    pub fn pipes(&self) -> &[PipeDefinition] {
        &self.pipes
    }

    pub fn contains(&self, identifier: &str) -> bool {
        self.pipes.iter().any(|pipe| pipe.identifier == identifier)
    }

    fn pipe(&self, identifier: &str) -> Result<&PipeDefinition, MoveError> {
        self.pipes
            .iter()
            .find(|pipe| pipe.identifier == identifier)
            .ok_or_else(|| MoveError::UnknownPipe(identifier.to_string()))
    }

    /// Build the domain [`Game`] this board describes.
    pub fn to_game(&self) -> Game {
        let mut pipes = HashMap::new();
        for definition in &self.pipes {
            pipes.insert(definition.identifier.clone(), definition.to_pipe());
        }
        Game::new(pipes)
    }

    /// Whether the rules consider this board finished.
    pub fn is_solved(&self) -> bool {
        self.to_game().is_finished()
    }

    /// How many pipes the rules consider completed.
    pub fn completed_pipes(&self) -> u8 {
        self.to_game().how_many_finished()
    }

    /// Whether the rules allow pouring `from` into `to` right now.
    pub fn can_play(&self, from: &str, to: &str) -> bool {
        match (self.pipe(from), self.pipe(to)) {
            (Ok(source), Ok(destination)) if from != to => {
                source.to_pipe().can_pour(&destination.to_pipe())
            }
            _ => false,
        }
    }

    /// Every move the rules allow on this board, in pipe order.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for source in &self.pipes {
            for destination in &self.pipes {
                if source.identifier == destination.identifier {
                    continue;
                }
                if source.to_pipe().can_pour(&destination.to_pipe()) {
                    moves.push(Move::new(&source.identifier, &destination.identifier));
                }
            }
        }
        moves
    }

    /// Play a pour and return the resulting board, leaving `self` untouched.
    ///
    /// Every single unit transfer is gated by `Pipe::can_pour`, exactly like
    /// `Pipe::pour_into` does: this method only mirrors the content so the new
    /// board can be observed, it never decides anything by itself.
    pub fn play(&self, movement: &Move) -> Result<Puzzle, MoveError> {
        let (from, to) = (movement.from.as_str(), movement.to.as_str());
        if from == to {
            return Err(MoveError::SamePipe(from.to_string()));
        }

        let mut source_colors = self.pipe(from)?.colors.clone();
        let mut destination_colors = self.pipe(to)?.colors.clone();

        let mut source = PipeDefinition::new(from, source_colors.clone()).to_pipe();
        let mut destination = PipeDefinition::new(to, destination_colors.clone()).to_pipe();

        if !source.can_pour(&destination) {
            return Err(MoveError::IllegalMove { from: from.to_string(), to: to.to_string() });
        }

        while source.can_pour(&destination) {
            let color = source_colors.pop().expect("can_pour guarantees a non empty source");
            destination_colors.push(color);
            source = PipeDefinition::new(from, source_colors.clone()).to_pipe();
            destination = PipeDefinition::new(to, destination_colors.clone()).to_pipe();
        }

        debug_assert_eq!(
            {
                let mut reference_source = self.pipe(from)?.to_pipe();
                let mut reference_destination = self.pipe(to)?.to_pipe();
                reference_source.pour_into(&mut reference_destination);
                (reference_source.get_filled_level(), reference_destination.get_filled_level())
            },
            (source_colors.len(), destination_colors.len()),
            "the mirrored pour diverged from Pipe::pour_into"
        );

        let pipes = self
            .pipes
            .iter()
            .map(|pipe| {
                if pipe.identifier == from {
                    PipeDefinition::new(from, source_colors.clone())
                } else if pipe.identifier == to {
                    PipeDefinition::new(to, destination_colors.clone())
                } else {
                    pipe.clone()
                }
            })
            .collect();

        Ok(Self { pipes })
    }

    /// Replay a sequence of moves from this board.
    pub fn play_all(&self, moves: &[Move]) -> Result<Puzzle, MoveError> {
        let mut board = self.clone();
        for movement in moves {
            board = board.play(movement)?;
        }
        Ok(board)
    }

    /// Render the board as `identifier -> color names`, in pipe order.
    pub fn to_named_pipes(&self) -> Vec<(String, Vec<&'static str>)> {
        self.pipes
            .iter()
            .map(|pipe| {
                (pipe.identifier.clone(), pipe.colors.iter().map(color_name).collect())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Puzzle {
        Puzzle::new(vec![
            PipeDefinition::new("P1", vec![Color::Red, Color::Blue]),
            PipeDefinition::new("P2", vec![Color::Blue, Color::Red]),
            PipeDefinition::new("P3", vec![]),
        ])
        .unwrap()
    }

    #[test]
    fn rejects_boards_the_rules_could_not_play() {
        assert_eq!(
            Puzzle::new(vec![PipeDefinition::new("P1", vec![])]).unwrap_err(),
            PuzzleError::NotEnoughPipes { found: 1 }
        );
        assert_eq!(
            Puzzle::new(vec![
                PipeDefinition::new("P1", vec![]),
                PipeDefinition::new("P1", vec![]),
            ])
            .unwrap_err(),
            PuzzleError::DuplicateIdentifier("P1".into())
        );
        assert!(matches!(
            Puzzle::new(vec![
                PipeDefinition::new(
                    "P1",
                    vec![Color::Red, Color::Red, Color::Red, Color::Red, Color::Red]
                ),
                PipeDefinition::new("P2", vec![]),
            ])
            .unwrap_err(),
            PuzzleError::PipeOverflow { .. }
        ));
    }

    #[test]
    fn playing_pours_every_unit_the_rules_allow() {
        let played = board().play(&Move::new("P1", "P3")).unwrap();

        assert_eq!(played.pipe("P1").unwrap().colors, vec![Color::Red]);
        assert_eq!(played.pipe("P3").unwrap().colors, vec![Color::Blue]);
        // the untouched pipe is preserved, and so is the pipe order
        assert_eq!(played.pipe("P2").unwrap().colors, vec![Color::Blue, Color::Red]);
        assert_eq!(
            played.pipes().iter().map(|p| p.identifier.as_str()).collect::<Vec<_>>(),
            vec!["P1", "P2", "P3"]
        );
    }

    #[test]
    fn stacks_matching_colors() {
        let matching_tops = Puzzle::new(vec![
            PipeDefinition::new("P1", vec![Color::Red, Color::Blue, Color::Blue]),
            PipeDefinition::new("P2", vec![Color::Red, Color::Blue]),
        ])
        .unwrap();

        let played = matching_tops.play(&Move::new("P1", "P2")).unwrap();

        // both units flow, because the destination keeps accepting blue
        assert_eq!(played.pipe("P1").unwrap().colors, vec![Color::Red]);
        assert_eq!(
            played.pipe("P2").unwrap().colors,
            vec![Color::Red, Color::Blue, Color::Blue, Color::Blue]
        );
    }

    #[test]
    fn refuses_moves_the_rules_reject() {
        let single_color = Puzzle::new(vec![
            PipeDefinition::new("P1", vec![Color::Red, Color::Red]),
            PipeDefinition::new("P2", vec![]),
        ])
        .unwrap();

        assert_eq!(
            single_color.play(&Move::new("P1", "P2")).unwrap_err(),
            MoveError::IllegalMove { from: "P1".into(), to: "P2".into() }
        );
        assert!(!single_color.can_play("P1", "P2"));
    }

    #[test]
    fn refuses_unknown_and_reflexive_moves() {
        assert_eq!(
            board().play(&Move::new("P1", "P9")).unwrap_err(),
            MoveError::UnknownPipe("P9".into())
        );
        assert_eq!(
            board().play(&Move::new("P1", "P1")).unwrap_err(),
            MoveError::SamePipe("P1".into())
        );
    }

    #[test]
    fn legal_moves_agree_with_can_play() {
        let board = board();
        for movement in board.legal_moves() {
            assert!(board.can_play(&movement.from, &movement.to));
            assert!(board.play(&movement).is_ok());
        }
    }

    #[test]
    fn detects_a_solved_board() {
        let solved = Puzzle::new(vec![
            PipeDefinition::new(
                "P1",
                vec![Color::Red, Color::Red, Color::Red, Color::Red],
            ),
            PipeDefinition::new("P2", vec![]),
        ])
        .unwrap();

        assert!(solved.is_solved());
        assert_eq!(solved.completed_pipes(), 2);
        assert!(!board().is_solved());
    }

    #[test]
    fn moves_round_trip_through_their_textual_form() {
        let movement = Move::new("P1", "P13");
        assert_eq!(movement.to_string(), "P1->P13");
        assert_eq!(Move::parse("P1->P13"), Some(movement));
        assert_eq!(Move::parse("P1"), None);
        assert_eq!(Move::parse("->P1"), None);
    }

    #[test]
    fn play_all_replays_a_solution() {
        let puzzle = Puzzle::new(vec![
            PipeDefinition::new("P1", vec![Color::Red, Color::Red, Color::Red, Color::Blue]),
            PipeDefinition::new("P2", vec![Color::Blue, Color::Blue, Color::Blue, Color::Red]),
            PipeDefinition::new("P3", vec![]),
        ])
        .unwrap();

        let moves: Vec<Move> = crate::solver::solve_puzzle_by_bruteforce(&puzzle.to_game())
            .expect("this board is solvable")
            .iter()
            .map(|raw| Move::parse(raw).expect("the solver renders parsable moves"))
            .collect();

        assert!(puzzle.play_all(&moves).unwrap().is_solved());
    }
}
