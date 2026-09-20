//! Wire format of a water sort board.
//!
//! The domain identifies a pipe by an opaque string; everything that leaves the
//! hexagon identifies it by a **UUID** and carries a human readable label next
//! to it. [`Board`] is that outside view, and it is the only place where the two
//! representations are bridged.
//!
//! Two JSON shapes are accepted, so the historical level files keep working:
//!
//! ```json
//! { "pipes": { "P1": ["Brown", "Lemon"], "P2": [] } }
//! ```
//!
//! ```json
//! { "pipes": [ { "label": "P1", "colors": ["Brown", "Lemon"] }, { "label": "P2", "colors": [] } ] }
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use water_sort_core::{Color, PipeDefinition, Puzzle, PuzzleError, color_name, parse_color};

/// A pipe as seen from outside the hexagon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardPipe {
    pub id: Uuid,
    pub label: String,
    /// Content of the pipe, bottom first.
    pub colors: Vec<String>,
}

/// An ordered board whose pipes are identified by UUID.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Board {
    pub pipes: Vec<BoardPipe>,
}

/// Why a submitted board could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    UnknownColor { pipe: String, color: String },
    DuplicateId(Uuid),
    Invalid(PuzzleError),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownColor { pipe, color } => {
                write!(f, "pipe '{pipe}' uses an unknown color '{color}'")
            }
            Self::DuplicateId(id) => write!(f, "duplicated pipe id '{id}'"),
            Self::Invalid(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for BoardError {}

impl From<PuzzleError> for BoardError {
    fn from(error: PuzzleError) -> Self {
        Self::Invalid(error)
    }
}

impl Board {
    /// Build a board, giving every pipe a fresh UUID when none was submitted.
    pub fn new(pipes: Vec<BoardPipe>) -> Result<Self, BoardError> {
        let mut seen: Vec<Uuid> = Vec::with_capacity(pipes.len());
        for pipe in &pipes {
            if seen.contains(&pipe.id) {
                return Err(BoardError::DuplicateId(pipe.id));
            }
            seen.push(pipe.id);
        }

        let board = Self { pipes };
        // Validated by the domain itself, so the two views cannot disagree.
        board.to_puzzle()?;
        Ok(board)
    }

    /// The domain view of this board: pipes keyed by their UUID.
    pub fn to_puzzle(&self) -> Result<Puzzle, BoardError> {
        let mut definitions = Vec::with_capacity(self.pipes.len());
        for pipe in &self.pipes {
            let mut colors = Vec::with_capacity(pipe.colors.len());
            for color in &pipe.colors {
                let parsed = parse_color(color).map_err(|_| BoardError::UnknownColor {
                    pipe: pipe.label.clone(),
                    color: color.clone(),
                })?;
                colors.push(parsed);
            }
            definitions.push(PipeDefinition::new(pipe.id.to_string(), colors));
        }
        Ok(Puzzle::new(definitions)?)
    }

    /// Same pipes, same labels, content taken from `puzzle`.
    pub fn with_puzzle(&self, puzzle: &Puzzle) -> Self {
        let pipes = self
            .pipes
            .iter()
            .map(|pipe| {
                let colors = puzzle
                    .pipes()
                    .iter()
                    .find(|definition| definition.identifier == pipe.id.to_string())
                    .map(|definition| render_colors(&definition.colors))
                    .unwrap_or_default();
                BoardPipe { id: pipe.id, label: pipe.label.clone(), colors }
            })
            .collect();
        Self { pipes }
    }

    pub fn contains(&self, id: Uuid) -> bool {
        self.pipes.iter().any(|pipe| pipe.id == id)
    }

    pub fn label_of(&self, id: Uuid) -> Option<&str> {
        self.pipes.iter().find(|pipe| pipe.id == id).map(|pipe| pipe.label.as_str())
    }

    pub fn id_of(&self, label: &str) -> Option<Uuid> {
        self.pipes.iter().find(|pipe| pipe.label == label).map(|pipe| pipe.id)
    }
}

fn render_colors(colors: &[Color]) -> Vec<String> {
    colors.iter().map(|color| color_name(color).to_string()).collect()
}

/// A pipe as submitted: the id and the label are optional.
#[derive(Debug, Clone, Deserialize)]
pub struct PipeDocument {
    #[serde(default)]
    pub id: Option<Uuid>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
}

/// A submitted board, in either of the two accepted shapes.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BoardDocument {
    List { pipes: Vec<PipeDocument> },
    /// Historical level file shape: `{"pipes": {"P1": ["Red", ...]}}`.
    Map { pipes: serde_json::Map<String, serde_json::Value> },
}

/// Fresh UUIDs, sorted so that the n-th one is the n-th smallest.
///
/// The solver walks the pipes in identifier order. Handing out sorted ids keeps
/// that walk aligned with the order the board was submitted in, so solving the
/// same board twice yields the same solution even though the ids differ.
fn generated_ids(count: usize) -> Vec<Uuid> {
    let mut ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();
    ids.sort();
    ids
}

impl BoardDocument {
    pub fn into_board(self) -> Result<Board, BoardError> {
        let pipes = match self {
            Self::List { pipes } => {
                let mut generated = generated_ids(pipes.len()).into_iter();
                pipes
                    .into_iter()
                    .enumerate()
                    .map(|(index, pipe)| BoardPipe {
                        id: pipe
                            .id
                            .or_else(|| generated.next())
                            .unwrap_or_else(Uuid::new_v4),
                        label: pipe.label.unwrap_or_else(|| format!("P{}", index + 1)),
                        colors: pipe.colors,
                    })
                    .collect()
            }
            Self::Map { pipes } => {
                let mut generated = generated_ids(pipes.len()).into_iter();
                let mut board_pipes = Vec::with_capacity(pipes.len());
                for (label, colors) in pipes {
                    let colors = colors
                        .as_array()
                        .ok_or_else(|| BoardError::UnknownColor {
                            pipe: label.clone(),
                            color: colors.to_string(),
                        })?
                        .iter()
                        .map(|color| match color.as_str() {
                            Some(name) => Ok(name.to_string()),
                            None => Err(BoardError::UnknownColor {
                                pipe: label.clone(),
                                color: color.to_string(),
                            }),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let id = generated.next().unwrap_or_else(Uuid::new_v4);
                    board_pipes.push(BoardPipe { id, label, colors });
                }
                board_pipes
            }
        };

        Board::new(pipes)
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BoardDocument::deserialize(deserializer)?
            .into_board()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use water_sort_core::Move;

    const MAP_FORM: &str = r#"{"pipes": {"P1": ["Red", "Blue"], "P2": ["Blue", "Red"], "P3": []}}"#;

    #[test]
    fn reads_the_historical_map_form() {
        let board: Board = serde_json::from_str(MAP_FORM).unwrap();

        assert_eq!(
            board.pipes.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(),
            vec!["P1", "P2", "P3"]
        );
        assert_eq!(board.pipes[0].colors, vec!["Red", "Blue"]);
        assert!(board.pipes.iter().all(|pipe| !pipe.id.is_nil()));
    }

    #[test]
    fn reads_the_list_form_and_keeps_submitted_ids() {
        let id = Uuid::new_v4();
        let raw = format!(
            r#"{{"pipes": [{{"id": "{id}", "label": "A", "colors": ["Red", "Blue"]}}, {{"colors": []}}]}}"#
        );

        let board: Board = serde_json::from_str(&raw).unwrap();

        assert_eq!(board.pipes[0].id, id);
        assert_eq!(board.pipes[0].label, "A");
        assert_eq!(board.pipes[1].label, "P2");
    }

    #[test]
    fn rejects_unknown_colors_and_duplicated_ids() {
        let raw = r#"{"pipes": {"P1": ["Chartreuse"], "P2": []}}"#;
        assert!(serde_json::from_str::<Board>(raw).is_err());

        let id = Uuid::new_v4();
        let raw = format!(
            r#"{{"pipes": [{{"id": "{id}", "colors": []}}, {{"id": "{id}", "colors": []}}]}}"#
        );
        assert!(serde_json::from_str::<Board>(&raw).is_err());
    }

    #[test]
    fn rejects_boards_the_domain_refuses() {
        let raw = r#"{"pipes": {"P1": ["Red", "Red", "Red", "Red", "Red"], "P2": []}}"#;
        assert!(serde_json::from_str::<Board>(raw).is_err());
    }

    #[test]
    fn projects_a_played_puzzle_back_onto_the_board() {
        let board: Board = serde_json::from_str(MAP_FORM).unwrap();
        let puzzle = board.to_puzzle().unwrap();
        let from = board.id_of("P1").unwrap();
        let to = board.id_of("P3").unwrap();

        let played = puzzle.play(&Move::new(from.to_string(), to.to_string())).unwrap();
        let updated = board.with_puzzle(&played);

        assert_eq!(updated.pipes[0].colors, vec!["Red"]);
        assert_eq!(updated.pipes[2].colors, vec!["Blue"]);
        assert_eq!(updated.pipes[1], board.pipes[1]);
        assert_eq!(board.label_of(from), Some("P1"));
    }

    #[test]
    fn generated_ids_follow_the_submitted_order() {
        let board: Board = serde_json::from_str(MAP_FORM).unwrap();
        let mut sorted = board.pipes.iter().map(|pipe| pipe.id).collect::<Vec<_>>();
        sorted.sort();

        assert_eq!(sorted, board.pipes.iter().map(|pipe| pipe.id).collect::<Vec<_>>());
    }

    #[test]
    fn serialized_boards_can_be_read_back() {
        let board: Board = serde_json::from_str(MAP_FORM).unwrap();
        let round_tripped: Board = serde_json::from_str(&serde_json::to_string(&board).unwrap()).unwrap();

        assert_eq!(round_tripped, board);
    }
}
