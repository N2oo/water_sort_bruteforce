//! What the API accepts and what it hands back.
//!
//! Every identifier crossing this boundary is a UUID: games, scenarios, moves
//! and pipes alike. Pipe labels travel next to the ids so a client can render a
//! board without keeping its own mapping.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use water_sort_format::Board;

use crate::application::{Solution, SolutionMove};
use crate::domain::{GameSession, PlayedMove, Scenario};

/// A slice of a listing, e.g. `?limit=20&offset=40`.
#[derive(Debug, Default, Deserialize)]
pub struct PageQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// The board itself, in either accepted shape.
    pub puzzle: Board,
}

#[derive(Debug, Deserialize)]
pub struct SolvePuzzleRequest {
    pub puzzle: Board,
}

/// A game starts either from a saved scenario or from a board submitted here.
#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    #[serde(default)]
    pub scenario_id: Option<Uuid>,
    #[serde(default)]
    pub puzzle: Option<Board>,
    #[serde(default)]
    pub name: Option<String>,
    /// When starting from a submitted board, also save it under this name.
    #[serde(default)]
    pub save_as_scenario: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveRequest {
    pub from: Uuid,
    pub to: Uuid,
}

/// One move, or a batch of them played in order.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PlayMovesRequest {
    One(MoveRequest),
    Many { moves: Vec<MoveRequest> },
}

impl PlayMovesRequest {
    pub fn into_moves(self) -> Vec<(Uuid, Uuid)> {
        match self {
            Self::One(movement) => vec![(movement.from, movement.to)],
            Self::Many { moves } => {
                moves.into_iter().map(|movement| (movement.from, movement.to)).collect()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RollBackRequest {
    /// How many moves to undo; defaults to the last one.
    #[serde(default)]
    pub steps: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct PipeResponse {
    pub id: Uuid,
    pub label: String,
    pub colors: Vec<String>,
}

fn pipes_of(board: &Board) -> Vec<PipeResponse> {
    board
        .pipes
        .iter()
        .map(|pipe| PipeResponse {
            id: pipe.id,
            label: pipe.label.clone(),
            colors: pipe.colors.clone(),
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ScenarioResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub pipes: Vec<PipeResponse>,
    pub created_at: DateTime<Utc>,
}

impl From<&Scenario> for ScenarioResponse {
    fn from(scenario: &Scenario) -> Self {
        Self {
            id: scenario.id,
            name: scenario.name.clone(),
            description: scenario.description.clone(),
            pipes: pipes_of(&scenario.board),
            created_at: scenario.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MoveResponse {
    pub id: Uuid,
    pub sequence: i32,
    pub from: Uuid,
    pub to: Uuid,
    pub from_label: Option<String>,
    pub to_label: Option<String>,
    pub notation: String,
    pub played_at: DateTime<Utc>,
    /// Board as it stood right after this move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_after: Option<Vec<PipeResponse>>,
}

impl MoveResponse {
    pub fn new(played: &PlayedMove, board: &Board, with_board: bool) -> Self {
        let from_label = board.label_of(played.from_pipe).map(str::to_string);
        let to_label = board.label_of(played.to_pipe).map(str::to_string);
        let notation = format!(
            "{}->{}",
            from_label.clone().unwrap_or_else(|| played.from_pipe.to_string()),
            to_label.clone().unwrap_or_else(|| played.to_pipe.to_string())
        );

        Self {
            id: played.id,
            sequence: played.sequence,
            from: played.from_pipe,
            to: played.to_pipe,
            from_label,
            to_label,
            notation,
            played_at: played.played_at,
            board_after: with_board.then(|| pipes_of(&played.board_after)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GameResponse {
    pub id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub name: Option<String>,
    pub status: String,
    pub solved: bool,
    pub moves_played: usize,
    pub completed_pipes: u8,
    pub pipes: Vec<PipeResponse>,
    pub initial_pipes: Vec<PipeResponse>,
    pub moves: Vec<MoveResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&GameSession> for GameResponse {
    fn from(session: &GameSession) -> Self {
        let completed_pipes = session
            .current_board
            .to_puzzle()
            .map(|puzzle| puzzle.completed_pipes())
            .unwrap_or(0);

        Self {
            id: session.id,
            scenario_id: session.scenario_id,
            name: session.name.clone(),
            status: session.status.as_str().to_string(),
            solved: session.is_solved(),
            moves_played: session.moves.len(),
            completed_pipes,
            pipes: pipes_of(&session.current_board),
            initial_pipes: pipes_of(&session.initial_board),
            moves: session
                .moves
                .iter()
                .map(|played| MoveResponse::new(played, &session.current_board, false))
                .collect(),
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

/// Answer of a `POST .../moves` call: what was played, and where the game stands.
#[derive(Debug, Serialize)]
pub struct PlayedMovesResponse {
    pub played: Vec<MoveResponse>,
    pub game: GameResponse,
}

/// Answer of a rollback: what was dropped, and where the game stands.
#[derive(Debug, Serialize)]
pub struct RollBackResponse {
    pub rolled_back: Vec<MoveResponse>,
    pub game: GameResponse,
}

#[derive(Debug, Serialize)]
pub struct SolutionMoveResponse {
    pub from: Uuid,
    pub to: Uuid,
    pub from_label: String,
    pub to_label: String,
    pub notation: String,
}

impl From<&SolutionMove> for SolutionMoveResponse {
    fn from(movement: &SolutionMove) -> Self {
        Self {
            from: movement.from_pipe,
            to: movement.to_pipe,
            from_label: movement.from_label.clone(),
            to_label: movement.to_label.clone(),
            notation: format!("{}->{}", movement.from_label, movement.to_label),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SolutionResponse {
    pub solved: bool,
    pub moves_count: usize,
    pub moves: Vec<SolutionMoveResponse>,
}

impl From<&Solution> for SolutionResponse {
    fn from(solution: &Solution) -> Self {
        Self {
            solved: solution.solved,
            moves_count: solution.moves.len(),
            moves: solution.moves.iter().map(SolutionMoveResponse::from).collect(),
        }
    }
}
