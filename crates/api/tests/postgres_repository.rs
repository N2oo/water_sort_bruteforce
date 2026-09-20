//! Storage tests against a real PostgreSQL.
//!
//! They are skipped unless `TEST_DATABASE_URL` points at a database the test
//! may migrate and write to, e.g.
//!
//! ```sh
//! TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1/water_sort cargo test -p water-sort-api
//! ```

use std::sync::{Arc, Mutex};

use serde_json::json;
use uuid::Uuid;
use water_sort_api::adapters::outbound::persistence::{
    PostgresGameSessionRepository, PostgresScenarioRepository, connect, migrate,
};
use water_sort_api::application::{GameSessionRepository, Page, ScenarioRepository};
use water_sort_api::domain::{GameSession, Scenario};
use water_sort_format::Board;

/// Each test owns its connection pool, because each one runs on its own tokio
/// runtime and a pool does not outlive the runtime that created it. Only the
/// migration is shared: several migrators racing on a fresh database fight over
/// the same tables.
static MIGRATED: Mutex<bool> = Mutex::new(false);

/// Open the test database, or tell the caller there is nothing to test against.
async fn repositories() -> Option<(Arc<dyn ScenarioRepository>, Arc<dyn GameSessionRepository>)> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let connection = connect(&url, 5).await.expect("cannot reach TEST_DATABASE_URL");

    {
        // Held across an await on purpose: every test thread drives its own
        // single task runtime, so blocking one while another migrates is safe.
        let mut migrated = MIGRATED.lock().unwrap_or_else(|error| error.into_inner());
        if !*migrated {
            migrate(&connection).await.expect("cannot migrate the test database");
            *migrated = true;
        }
    }

    Some((
        Arc::new(PostgresScenarioRepository::new(connection.clone())),
        Arc::new(PostgresGameSessionRepository::new(connection)),
    ))
}

macro_rules! repositories_or_skip {
    () => {
        match repositories().await {
            Some(repositories) => repositories,
            None => {
                eprintln!("TEST_DATABASE_URL is not set, skipping");
                return;
            }
        }
    };
}

fn board() -> Board {
    serde_json::from_value(json!({
        "pipes": { "P1": ["Red", "Red", "Red", "Blue"], "P2": ["Blue", "Blue", "Blue", "Red"], "P3": [] }
    }))
    .unwrap()
}

#[tokio::test]
async fn a_scenario_survives_a_round_trip() {
    let (scenarios, _) = repositories_or_skip!();
    let scenario = Scenario::new("round trip".into(), Some("saved board".into()), board());

    scenarios.save(&scenario).await.unwrap();
    let reloaded = scenarios.find(scenario.id).await.unwrap().expect("just saved");

    assert_eq!(reloaded.id, scenario.id);
    assert_eq!(reloaded.name, "round trip");
    assert_eq!(reloaded.description.as_deref(), Some("saved board"));
    assert_eq!(reloaded.board, scenario.board);
    // pipe uuids are stored as they were handed out
    assert_eq!(reloaded.board.pipes[0].id, scenario.board.pipes[0].id);

    assert!(scenarios.list(Page::default()).await.unwrap().iter().any(|s| s.id == scenario.id));
    assert!(scenarios.delete(scenario.id).await.unwrap());
    assert!(scenarios.find(scenario.id).await.unwrap().is_none());
    assert!(!scenarios.delete(scenario.id).await.unwrap());
}

#[tokio::test]
async fn a_game_and_its_history_survive_a_round_trip() {
    let (scenarios, sessions) = repositories_or_skip!();
    let scenario = Scenario::new("played".into(), None, board());
    scenarios.save(&scenario).await.unwrap();

    let mut session =
        GameSession::start(scenario.board.clone(), Some(scenario.id), Some("try".into())).unwrap();
    let p1 = session.current_board.id_of("P1").unwrap();
    let p2 = session.current_board.id_of("P2").unwrap();
    let p3 = session.current_board.id_of("P3").unwrap();
    session.play(p1, p3).unwrap();
    session.play(p2, p1).unwrap();
    sessions.save(&session).await.unwrap();

    let reloaded = sessions.find(session.id).await.unwrap().expect("just saved");

    assert_eq!(reloaded.scenario_id, Some(scenario.id));
    assert_eq!(reloaded.name.as_deref(), Some("try"));
    assert_eq!(reloaded.status, session.status);
    assert_eq!(reloaded.initial_board, session.initial_board);
    assert_eq!(reloaded.current_board, session.current_board);
    assert_eq!(reloaded.moves.len(), 2);
    assert_eq!(
        reloaded.moves.iter().map(|played| played.sequence).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(reloaded.moves[0].id, session.moves[0].id);
    assert_eq!(reloaded.moves[1].board_after, session.moves[1].board_after);

    sessions.delete(session.id).await.unwrap();
    scenarios.delete(scenario.id).await.unwrap();
}

#[tokio::test]
async fn a_rolled_back_move_leaves_the_storage() {
    let (_, sessions) = repositories_or_skip!();
    let mut session = GameSession::start(board(), None, None).unwrap();
    let p1 = session.current_board.id_of("P1").unwrap();
    let p2 = session.current_board.id_of("P2").unwrap();
    let p3 = session.current_board.id_of("P3").unwrap();
    session.play(p1, p3).unwrap();
    session.play(p2, p1).unwrap();
    sessions.save(&session).await.unwrap();

    let dropped = session.roll_back(1).unwrap();
    sessions.save(&session).await.unwrap();

    let reloaded = sessions.find(session.id).await.unwrap().unwrap();
    assert_eq!(reloaded.moves.len(), 1);
    assert!(
        reloaded.moves.iter().all(|played| played.id != dropped[0].id),
        "the rolled back move must be gone from the storage"
    );
    assert_eq!(reloaded.current_board, session.current_board);

    // replaying reuses the freed sequence number, which the unique index guards
    let replayed = session.play(p2, p1).unwrap().clone();
    assert_eq!(replayed.sequence, 2);
    sessions.save(&session).await.unwrap();
    assert_eq!(sessions.find(session.id).await.unwrap().unwrap().moves.len(), 2);

    sessions.delete(session.id).await.unwrap();
}

#[tokio::test]
async fn listing_returns_the_history_of_each_game() {
    let (_, sessions) = repositories_or_skip!();
    let mut first = GameSession::start(board(), None, Some("first".into())).unwrap();
    let p1 = first.current_board.id_of("P1").unwrap();
    let p3 = first.current_board.id_of("P3").unwrap();
    first.play(p1, p3).unwrap();
    let second = GameSession::start(board(), None, Some("second".into())).unwrap();
    sessions.save(&first).await.unwrap();
    sessions.save(&second).await.unwrap();

    let listed = sessions.list(Page::new(Some(100), None)).await.unwrap();

    let reloaded_first = listed.iter().find(|session| session.id == first.id).unwrap();
    let reloaded_second = listed.iter().find(|session| session.id == second.id).unwrap();
    assert_eq!(reloaded_first.moves.len(), 1);
    assert!(reloaded_second.moves.is_empty());

    sessions.delete(first.id).await.unwrap();
    sessions.delete(second.id).await.unwrap();
}

#[tokio::test]
async fn an_unknown_id_is_simply_absent() {
    let (scenarios, sessions) = repositories_or_skip!();
    let unknown = Uuid::new_v4();

    assert!(scenarios.find(unknown).await.unwrap().is_none());
    assert!(sessions.find(unknown).await.unwrap().is_none());
}
