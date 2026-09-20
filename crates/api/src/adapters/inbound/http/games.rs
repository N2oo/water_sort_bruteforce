//! `/api/v1/games`: running games, their history and their solution.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::AppState;
use super::dto::{
    CreateGameRequest, GameResponse, MoveResponse, PageQuery, PlayMovesRequest,
    PlayedMovesResponse, RollBackRequest, RollBackResponse, SolutionResponse,
};
use super::error::{ApiError, ApiResult};
use crate::application::{Page, StartGame};

pub async fn create(
    State(state): State<AppState>,
    payload: Result<Json<CreateGameRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<GameResponse>)> {
    let Json(payload) = payload?;

    let request = match (payload.scenario_id, payload.puzzle) {
        (Some(_), Some(_)) => {
            return Err(ApiError::bad_request(
                "submit either 'scenario_id' or 'puzzle', not both",
            ));
        }
        (None, None) => {
            return Err(ApiError::bad_request(
                "a game needs either a 'scenario_id' or a 'puzzle'",
            ));
        }
        (Some(scenario_id), None) => StartGame::FromScenario { scenario_id, name: payload.name },
        (None, Some(puzzle)) => StartGame::FromBoard {
            board: puzzle,
            name: payload.name,
            save_as_scenario: payload.save_as_scenario,
        },
    };

    let session = state.games.start(request).await?;
    Ok((StatusCode::CREATED, Json(GameResponse::from(&session))))
}

pub async fn list(
    State(state): State<AppState>,
    Query(page): Query<PageQuery>,
) -> ApiResult<Json<Vec<GameResponse>>> {
    let sessions = state.games.list(Page::new(page.limit, page.offset)).await?;
    Ok(Json(sessions.iter().map(GameResponse::from).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<GameResponse>> {
    let session = state.games.get(id).await?;
    Ok(Json(GameResponse::from(&session)))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<StatusCode> {
    state.games.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The history of a game, board snapshots included.
pub async fn moves(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<MoveResponse>>> {
    let session = state.games.get(id).await?;
    Ok(Json(
        session
            .moves
            .iter()
            .map(|played| MoveResponse::new(played, &session.current_board, true))
            .collect(),
    ))
}

/// Play one move, or a batch of them. A refused move leaves the game untouched.
pub async fn play(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    payload: Result<Json<PlayMovesRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<PlayedMovesResponse>)> {
    let Json(payload) = payload?;
    let moves = payload.into_moves();
    if moves.is_empty() {
        return Err(ApiError::bad_request("submit at least one move"));
    }

    let (session, played) = state.games.play_all(id, &moves).await?;
    let response = PlayedMovesResponse {
        played: played
            .iter()
            .map(|movement| MoveResponse::new(movement, &session.current_board, false))
            .collect(),
        game: GameResponse::from(&session),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Undo the last `steps` moves; they are dropped from the history for good.
pub async fn roll_back(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    payload: Option<Json<RollBackRequest>>,
) -> ApiResult<Json<RollBackResponse>> {
    // No body at all means "undo the last move".
    let steps = payload.and_then(|Json(payload)| payload.steps).unwrap_or(1);

    let (session, dropped) = state.games.roll_back(id, steps).await?;
    Ok(Json(RollBackResponse {
        rolled_back: dropped
            .iter()
            .map(|movement| MoveResponse::new(movement, &session.current_board, false))
            .collect(),
        game: GameResponse::from(&session),
    }))
}

/// Undo everything, back to the board the game started from.
pub async fn reset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RollBackResponse>> {
    let (session, dropped) = state.games.reset(id).await?;
    Ok(Json(RollBackResponse {
        rolled_back: dropped
            .iter()
            .map(|movement| MoveResponse::new(movement, &session.current_board, false))
            .collect(),
        game: GameResponse::from(&session),
    }))
}

/// Solve the game **from where it currently stands**: the returned moves can be
/// posted back to `/moves` as is.
pub async fn solve(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SolutionResponse>> {
    let solution = state.solving.solve_session(id).await?;
    Ok(Json(SolutionResponse::from(&solution)))
}
