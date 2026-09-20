//! `/api/v1/puzzles`: the stateless side of the API.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;

use super::AppState;
use super::dto::{SolutionResponse, SolvePuzzleRequest};
use super::error::ApiResult;

/// Solve a board submitted on the spot, without storing anything.
pub async fn solve(
    State(state): State<AppState>,
    payload: Result<Json<SolvePuzzleRequest>, JsonRejection>,
) -> ApiResult<Json<SolutionResponse>> {
    let Json(payload) = payload?;
    let solution = state.solving.solve_board(&payload.puzzle).await?;
    Ok(Json(SolutionResponse::from(&solution)))
}
