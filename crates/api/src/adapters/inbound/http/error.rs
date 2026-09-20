//! One error envelope for the whole API.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::application::{ApplicationError, RepositoryError, SolverError};
use crate::domain::DomainError;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

/// Anything an HTTP handler can answer with instead of a payload.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into() }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse { error: ErrorBody { code: self.code, message: self.message } };
        (self.status, Json(body)).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_payload", rejection.body_text())
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::NotFound { resource, id } => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("no {resource} with id '{id}'"),
            ),
            ApplicationError::Invalid(message) => Self::bad_request(message),
            ApplicationError::Domain(domain) => domain.into(),
            ApplicationError::Solver(solver) => solver.into(),
            ApplicationError::Repository(repository) => repository.into(),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        let (status, code) = match error {
            DomainError::UnknownPipe(_) => (StatusCode::UNPROCESSABLE_ENTITY, "unknown_pipe"),
            DomainError::SamePipe(_) => (StatusCode::UNPROCESSABLE_ENTITY, "same_pipe"),
            DomainError::IllegalMove { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "illegal_move"),
            DomainError::AlreadySolved => (StatusCode::CONFLICT, "already_solved"),
            DomainError::NotEnoughMoves { .. } => (StatusCode::CONFLICT, "not_enough_moves"),
            DomainError::NothingToRollBack => (StatusCode::CONFLICT, "nothing_to_roll_back"),
            DomainError::CorruptedBoard(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "corrupted_board")
            }
        };

        Self::new(status, code, error.to_string())
    }
}

impl From<SolverError> for ApiError {
    fn from(error: SolverError) -> Self {
        let (status, code) = match error {
            SolverError::InvalidBoard(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_board"),
            SolverError::TimedOut { .. } => (StatusCode::GATEWAY_TIMEOUT, "solver_timeout"),
            SolverError::Crashed(_) => (StatusCode::INTERNAL_SERVER_ERROR, "solver_failure"),
        };

        Self::new(status, code, error.to_string())
    }
}

impl From<RepositoryError> for ApiError {
    fn from(error: RepositoryError) -> Self {
        tracing::error!(%error, "storage failure");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_failure",
            "the request could not be stored".to_string(),
        )
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
