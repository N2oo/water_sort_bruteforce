//! `/api/v1/scenarios`: boards saved once, replayed as often as wanted.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::AppState;
use super::dto::{
    CreateScenarioRequest, PageQuery, ScenarioResponse, SolutionResponse,
};
use super::error::ApiResult;
use crate::application::Page;

pub async fn create(
    State(state): State<AppState>,
    payload: Result<Json<CreateScenarioRequest>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<ScenarioResponse>)> {
    let Json(payload) = payload?;
    let scenario = state
        .scenarios
        .create(payload.name, payload.description, payload.puzzle)
        .await?;

    Ok((StatusCode::CREATED, Json(ScenarioResponse::from(&scenario))))
}

pub async fn list(
    State(state): State<AppState>,
    Query(page): Query<PageQuery>,
) -> ApiResult<Json<Vec<ScenarioResponse>>> {
    let scenarios = state.scenarios.list(Page::new(page.limit, page.offset)).await?;
    Ok(Json(scenarios.iter().map(ScenarioResponse::from).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ScenarioResponse>> {
    let scenario = state.scenarios.get(id).await?;
    Ok(Json(ScenarioResponse::from(&scenario)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.scenarios.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Solve a saved scenario, from its initial board.
pub async fn solve(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SolutionResponse>> {
    let solution = state.solving.solve_scenario(id).await?;
    Ok(Json(SolutionResponse::from(&solution)))
}
