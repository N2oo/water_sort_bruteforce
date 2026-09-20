//! Driving adapter: the HTTP API.
//!
//! Handlers do no game logic at all; they translate a request into a use case
//! call and a use case answer into JSON.

pub mod dto;
pub mod error;
mod games;
mod puzzles;
mod scenarios;

use std::sync::Arc;

use axum::Json;
use axum::http::{Method, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Router, serve::Serve};
use serde_json::json;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::application::{
    GameService, GameSessionRepository, PuzzleSolver, ScenarioRepository, ScenarioService,
    SolvingService,
};
use crate::config::CorsOrigins;

/// What every handler is given: the use cases, nothing else.
#[derive(Clone)]
pub struct AppState {
    pub scenarios: ScenarioService,
    pub games: GameService,
    pub solving: SolvingService,
}

impl AppState {
    /// Wire the use cases onto a set of adapters.
    pub fn new(
        scenario_repository: Arc<dyn ScenarioRepository>,
        session_repository: Arc<dyn GameSessionRepository>,
        solver: Arc<dyn PuzzleSolver>,
    ) -> Self {
        Self {
            scenarios: ScenarioService::new(scenario_repository.clone()),
            games: GameService::new(session_repository.clone(), scenario_repository.clone()),
            solving: SolvingService::new(solver, session_repository, scenario_repository),
        }
    }
}

/// Every route of the API.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/api/v1/puzzles/solve", post(puzzles::solve))
        .route("/api/v1/scenarios", get(scenarios::list).post(scenarios::create))
        .route("/api/v1/scenarios/{id}", get(scenarios::get).delete(scenarios::delete))
        .route("/api/v1/scenarios/{id}/solve", post(scenarios::solve))
        .route("/api/v1/games", get(games::list).post(games::create))
        .route("/api/v1/games/{id}", get(games::get))
        .route("/api/v1/games/{id}", delete(games::delete))
        .route("/api/v1/games/{id}/moves", get(games::moves).post(games::play))
        .route("/api/v1/games/{id}/rollback", post(games::roll_back))
        .route("/api/v1/games/{id}/reset", post(games::reset))
        .route("/api/v1/games/{id}/solve", post(games::solve))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// The browser front end is served from its own origin, so it needs this.
///
/// Nothing here is authenticated — no cookie, no credentials — so allowing any
/// origin hands a page nothing it could not already fetch from the API itself.
/// `CORS_ALLOWED_ORIGINS` narrows it to a list when a deployment wants that.
pub fn cors(origins: &CorsOrigins) -> CorsLayer {
    let allowed = match origins {
        CorsOrigins::Any => AllowOrigin::any(),
        CorsOrigins::List(list) => AllowOrigin::list(
            list.iter().filter_map(|origin| origin.parse().ok()).collect::<Vec<_>>(),
        ),
    };

    CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// A short map of the API, handy when poking at it by hand.
async fn index() -> impl IntoResponse {
    Json(json!({
        "name": "water-sort-api",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "POST   /api/v1/puzzles/solve": "solve a submitted puzzle, storing nothing",
            "GET    /api/v1/scenarios": "list the saved puzzles",
            "POST   /api/v1/scenarios": "save a puzzle under a name",
            "GET    /api/v1/scenarios/{id}": "read a saved puzzle",
            "DELETE /api/v1/scenarios/{id}": "drop a saved puzzle",
            "POST   /api/v1/scenarios/{id}/solve": "solve a saved puzzle",
            "GET    /api/v1/games": "list the game sessions",
            "POST   /api/v1/games": "start a game from a scenario or a submitted puzzle",
            "GET    /api/v1/games/{id}": "read a game session",
            "DELETE /api/v1/games/{id}": "drop a game session",
            "GET    /api/v1/games/{id}/moves": "read the played moves",
            "POST   /api/v1/games/{id}/moves": "play one move or a batch of moves",
            "POST   /api/v1/games/{id}/rollback": "undo the last moves and drop them",
            "POST   /api/v1/games/{id}/reset": "undo every move",
            "POST   /api/v1/games/{id}/solve": "solve the game from its current state"
        }
    }))
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": { "code": "not_found", "message": "unknown endpoint" } })),
    )
}

/// Serve `router` until the process is asked to stop.
pub async fn serve(
    listener: tokio::net::TcpListener,
    router: Router,
) -> std::io::Result<()> {
    let server: Serve<_, _, _> = axum::serve(listener, router);
    server.with_graceful_shutdown(shutdown_signal()).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("cannot listen to ctrl-c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("cannot listen to SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutting down");
}
