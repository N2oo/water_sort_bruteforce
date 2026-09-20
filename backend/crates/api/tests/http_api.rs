//! End to end tests of the driving adapter, wired onto the in-memory storage.
//!
//! They exercise the API exactly like a client would: JSON in, JSON out.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use water_sort_api::adapters::inbound::http::{AppState, cors, router};
use water_sort_api::adapters::outbound::persistence::{
    InMemoryGameSessionRepository, InMemoryScenarioRepository,
};
use water_sort_api::adapters::outbound::solver::BruteforceSolver;
use water_sort_api::config::CorsOrigins;

fn api() -> Router {
    router(AppState::new(
        Arc::new(InMemoryScenarioRepository::default()),
        Arc::new(InMemoryGameSessionRepository::default()),
        Arc::new(BruteforceSolver::new(Duration::from_secs(60), 64 * 1024 * 1024)),
    ))
}

async fn call(api: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(body) => request
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => request.body(Body::empty()).unwrap(),
    };

    let response = api.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let payload = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("the API always answers JSON")
    };

    (status, payload)
}

/// A three pipe board that can be solved in a couple of moves.
fn puzzle() -> Value {
    json!({
        "pipes": {
            "P1": ["Red", "Red", "Red", "Blue"],
            "P2": ["Blue", "Blue", "Blue", "Red"],
            "P3": []
        }
    })
}

fn pipe_id(game: &Value, label: &str) -> Uuid {
    game["pipes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pipe| pipe["label"] == label)
        .map(|pipe| pipe["id"].as_str().unwrap().parse().unwrap())
        .unwrap_or_else(|| panic!("no pipe labelled {label}"))
}

fn colors(game: &Value, label: &str) -> Vec<String> {
    game["pipes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pipe| pipe["label"] == label)
        .unwrap()["colors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|color| color.as_str().unwrap().to_string())
        .collect()
}

async fn start_game(api: &Router) -> Value {
    let (status, game) =
        call(api, "POST", "/api/v1/games", Some(json!({ "puzzle": puzzle() }))).await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    game
}

#[tokio::test]
async fn health_is_public() {
    let (status, body) = call(&api(), "GET", "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn a_scenario_can_be_saved_listed_and_replayed() {
    let api = api();

    let (status, scenario) = call(
        &api,
        "POST",
        "/api/v1/scenarios",
        Some(json!({ "name": "Level 145", "description": "from the level files", "puzzle": puzzle() })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{scenario}");
    let scenario_id = scenario["id"].as_str().unwrap().to_string();
    assert_eq!(scenario["pipes"].as_array().unwrap().len(), 3);
    assert!(Uuid::parse_str(&scenario_id).is_ok());
    // every pipe carries a uuid of its own
    assert!(
        scenario["pipes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|pipe| Uuid::parse_str(pipe["id"].as_str().unwrap()).is_ok())
    );

    let (status, listed) = call(&api, "GET", "/api/v1/scenarios", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let (status, game) = call(
        &api,
        "POST",
        "/api/v1/games",
        Some(json!({ "scenario_id": scenario_id, "name": "first try" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    assert_eq!(game["scenario_id"], scenario_id);
    assert_eq!(game["status"], "running");
    assert_eq!(game["moves_played"], 0);
}

#[tokio::test]
async fn a_game_can_be_started_from_a_submitted_puzzle_and_saved_as_a_scenario() {
    let api = api();

    let (status, game) = call(
        &api,
        "POST",
        "/api/v1/games",
        Some(json!({ "puzzle": puzzle(), "save_as_scenario": "kept for later" })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{game}");
    let scenario_id = game["scenario_id"].as_str().expect("the board was saved");

    let (status, scenario) =
        call(&api, "GET", &format!("/api/v1/scenarios/{scenario_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(scenario["name"], "kept for later");
}

#[tokio::test]
async fn playing_a_move_updates_the_board_and_the_history() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P3"));

    let (status, played) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p1, "to": p3 })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{played}");
    assert_eq!(played["played"][0]["sequence"], 1);
    assert_eq!(played["played"][0]["notation"], "P1->P3");
    assert!(Uuid::parse_str(played["played"][0]["id"].as_str().unwrap()).is_ok());
    assert_eq!(played["game"]["moves_played"], 1);
    assert_eq!(colors(&played["game"], "P1"), vec!["Red", "Red", "Red"]);
    assert_eq!(colors(&played["game"], "P3"), vec!["Blue"]);

    let (status, stored) = call(&api, "GET", &format!("/api/v1/games/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stored["moves_played"], 1);

    let (status, moves) = call(&api, "GET", &format!("/api/v1/games/{id}/moves"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(moves.as_array().unwrap().len(), 1);
    assert!(moves[0]["board_after"].is_array(), "the history keeps the resulting board");
}

#[tokio::test]
async fn several_moves_can_be_played_in_one_call() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p2, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P2"), pipe_id(&game, "P3"));

    let (status, played) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "moves": [{ "from": p1, "to": p3 }, { "from": p2, "to": p1 }] })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{played}");
    assert_eq!(played["played"].as_array().unwrap().len(), 2);
    assert_eq!(played["game"]["moves_played"], 2);
}

#[tokio::test]
async fn a_refused_move_leaves_the_game_untouched() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p2, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P2"), pipe_id(&game, "P3"));

    // P1 ends with blue, P2 with red: the rules refuse this pour
    let (status, error) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p1, "to": p2 })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
    assert_eq!(error["error"]["code"], "illegal_move");

    // the second move of the batch is illegal, so the first one is not kept either
    let (status, _) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "moves": [{ "from": p1, "to": p3 }, { "from": p1, "to": p2 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (_, stored) = call(&api, "GET", &format!("/api/v1/games/{id}"), None).await;
    assert_eq!(stored["moves_played"], 0);
    assert_eq!(colors(&stored, "P1"), vec!["Red", "Red", "Red", "Blue"]);
}

#[tokio::test]
async fn rolling_back_restores_the_board_and_drops_the_moves() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p2, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P2"), pipe_id(&game, "P3"));

    call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "moves": [{ "from": p1, "to": p3 }, { "from": p2, "to": p1 }] })),
    )
    .await;

    let (status, rolled) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/rollback"),
        Some(json!({ "steps": 1 })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{rolled}");
    assert_eq!(rolled["rolled_back"].as_array().unwrap().len(), 1);
    assert_eq!(rolled["rolled_back"][0]["sequence"], 2);
    assert_eq!(rolled["game"]["moves_played"], 1);
    assert_eq!(colors(&rolled["game"], "P3"), vec!["Blue"]);

    // the dropped move is gone from the history for good
    let (_, moves) = call(&api, "GET", &format!("/api/v1/games/{id}/moves"), None).await;
    assert_eq!(moves.as_array().unwrap().len(), 1);
    assert_eq!(moves[0]["sequence"], 1);

    // and the next move takes its place
    let (status, played) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p2, "to": p1 })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{played}");
    assert_eq!(played["played"][0]["sequence"], 2);
}

#[tokio::test]
async fn rolling_back_without_a_body_undoes_the_last_move() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P3"));
    call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p1, "to": p3 })),
    )
    .await;

    let (status, rolled) = call(&api, "POST", &format!("/api/v1/games/{id}/rollback"), None).await;

    assert_eq!(status, StatusCode::OK, "{rolled}");
    assert_eq!(rolled["game"]["moves_played"], 0);
    assert_eq!(colors(&rolled["game"], "P1"), vec!["Red", "Red", "Red", "Blue"]);
}

#[tokio::test]
async fn resetting_undoes_every_move() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p2, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P2"), pipe_id(&game, "P3"));
    call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "moves": [{ "from": p1, "to": p3 }, { "from": p2, "to": p1 }] })),
    )
    .await;

    let (status, rolled) = call(&api, "POST", &format!("/api/v1/games/{id}/reset"), None).await;

    assert_eq!(status, StatusCode::OK, "{rolled}");
    assert_eq!(rolled["rolled_back"].as_array().unwrap().len(), 2);
    assert_eq!(rolled["game"]["moves_played"], 0);
    assert_eq!(rolled["game"]["pipes"], game["pipes"]);
}

#[tokio::test]
async fn rolling_back_more_than_what_was_played_is_refused() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();

    let (status, error) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/rollback"),
        Some(json!({ "steps": 4 })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["code"], "not_enough_moves");
}

#[tokio::test]
async fn a_submitted_puzzle_can_be_solved_without_being_stored() {
    let api = api();

    let (status, solution) =
        call(&api, "POST", "/api/v1/puzzles/solve", Some(json!({ "puzzle": puzzle() }))).await;

    assert_eq!(status, StatusCode::OK, "{solution}");
    assert_eq!(solution["solved"], true);
    assert!(solution["moves_count"].as_u64().unwrap() > 0);
    assert!(solution["moves"][0]["notation"].as_str().unwrap().contains("->"));

    let (_, games) = call(&api, "GET", "/api/v1/games", None).await;
    assert!(games.as_array().unwrap().is_empty(), "solving stores nothing");
}

#[tokio::test]
async fn a_running_game_is_solved_from_its_current_state() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p3) = (pipe_id(&game, "P1"), pipe_id(&game, "P3"));

    let (_, before) = call(&api, "POST", &format!("/api/v1/games/{id}/solve"), None).await;
    call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p1, "to": p3 })),
    )
    .await;
    let (status, after) = call(&api, "POST", &format!("/api/v1/games/{id}/solve"), None).await;

    assert_eq!(status, StatusCode::OK, "{after}");
    assert_eq!(after["solved"], true);
    assert_eq!(
        after["moves_count"].as_u64().unwrap(),
        before["moves_count"].as_u64().unwrap() - 1,
        "one move was played, so one move less is left"
    );
}

#[tokio::test]
async fn the_solution_of_a_game_can_be_played_back() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();

    let (_, solution) = call(&api, "POST", &format!("/api/v1/games/{id}/solve"), None).await;
    let moves: Vec<Value> = solution["moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|movement| json!({ "from": movement["from"], "to": movement["to"] }))
        .collect();

    let (status, played) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "moves": moves })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{played}");
    assert_eq!(played["game"]["status"], "solved");
    assert_eq!(played["game"]["solved"], true);
}

#[tokio::test]
async fn a_solved_game_refuses_further_moves() {
    let api = api();
    let (_, game) = call(
        &api,
        "POST",
        "/api/v1/games",
        Some(json!({ "puzzle": { "pipes": { "P1": ["Red", "Red", "Red"], "P2": ["Red"], "P3": [] } } })),
    )
    .await;
    let id = game["id"].as_str().unwrap().to_string();
    let (p1, p2) = (pipe_id(&game, "P1"), pipe_id(&game, "P2"));

    call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p2, "to": p1 })),
    )
    .await;

    let (status, error) = call(
        &api,
        "POST",
        &format!("/api/v1/games/{id}/moves"),
        Some(json!({ "from": p1, "to": p2 })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["code"], "already_solved");
}

#[tokio::test]
async fn unknown_ids_and_malformed_requests_are_rejected() {
    let api = api();
    let unknown = Uuid::new_v4();

    let (status, error) = call(&api, "GET", &format!("/api/v1/games/{unknown}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error["error"]["code"], "not_found");

    let (status, _) = call(&api, "GET", &format!("/api/v1/scenarios/{unknown}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, error) = call(&api, "POST", "/api/v1/games", Some(json!({}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{error}");
    assert_eq!(error["error"]["code"], "bad_request");

    let (status, _) = call(
        &api,
        "POST",
        "/api/v1/games",
        Some(json!({ "scenario_id": unknown, "puzzle": puzzle() })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = call(
        &api,
        "POST",
        "/api/v1/games",
        Some(json!({ "puzzle": { "pipes": { "P1": ["Chartreuse"], "P2": [] } } })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unknown colors are refused");

    let (status, _) = call(&api, "GET", "/api/v1/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_game_and_a_scenario_can_be_dropped() {
    let api = api();
    let game = start_game(&api).await;
    let id = game["id"].as_str().unwrap().to_string();

    let (status, _) = call(&api, "DELETE", &format!("/api/v1/games/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(&api, "GET", &format!("/api/v1/games/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The browser front end is served from its own origin, so the preflight of a
/// `POST` has to come back with the headers that let the call through.
#[tokio::test]
async fn answers_the_cors_preflight_of_the_front_end() {
    let origin = "http://localhost:5173";
    let api = api().layer(cors(&CorsOrigins::List(vec![origin.to_string()])));

    let preflight = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/scenarios")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();

    let response = api.clone().oneshot(preflight).await.unwrap();
    let headers = response.headers();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(headers.get("access-control-allow-origin").unwrap(), origin);
    assert!(
        headers
            .get("access-control-allow-methods")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("POST")
    );

    // An origin that was not allowed gets no header, and the browser stops there.
    let refused = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/scenarios")
        .header("origin", "http://evil.example")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = api.oneshot(refused).await.unwrap();
    assert!(response.headers().get("access-control-allow-origin").is_none());
}
