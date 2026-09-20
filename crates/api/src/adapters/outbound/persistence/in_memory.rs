//! Driven adapter: an in-memory storage.
//!
//! Same ports, no database. It backs the HTTP tests and makes it possible to
//! boot the API without Postgres when trying things out.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::{GameSessionRepository, Page, RepositoryError, ScenarioRepository};
use crate::domain::{GameSession, Scenario};

#[derive(Default)]
pub struct InMemoryScenarioRepository {
    rows: Mutex<HashMap<Uuid, Scenario>>,
}

#[async_trait]
impl ScenarioRepository for InMemoryScenarioRepository {
    async fn save(&self, scenario: &Scenario) -> Result<(), RepositoryError> {
        self.rows.lock().unwrap().insert(scenario.id, scenario.clone());
        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Option<Scenario>, RepositoryError> {
        Ok(self.rows.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: Page) -> Result<Vec<Scenario>, RepositoryError> {
        let mut rows: Vec<Scenario> = self.rows.lock().unwrap().values().cloned().collect();
        rows.sort_by(|left, right| right.created_at.cmp(&left.created_at).then(right.id.cmp(&left.id)));
        Ok(paginate(rows, page))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        Ok(self.rows.lock().unwrap().remove(&id).is_some())
    }
}

#[derive(Default)]
pub struct InMemoryGameSessionRepository {
    rows: Mutex<HashMap<Uuid, GameSession>>,
}

#[async_trait]
impl GameSessionRepository for InMemoryGameSessionRepository {
    async fn save(&self, session: &GameSession) -> Result<(), RepositoryError> {
        self.rows.lock().unwrap().insert(session.id, session.clone());
        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Option<GameSession>, RepositoryError> {
        Ok(self.rows.lock().unwrap().get(&id).cloned())
    }

    async fn list(&self, page: Page) -> Result<Vec<GameSession>, RepositoryError> {
        let mut rows: Vec<GameSession> = self.rows.lock().unwrap().values().cloned().collect();
        rows.sort_by(|left, right| right.created_at.cmp(&left.created_at).then(right.id.cmp(&left.id)));
        Ok(paginate(rows, page))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        Ok(self.rows.lock().unwrap().remove(&id).is_some())
    }
}

fn paginate<T>(rows: Vec<T>, page: Page) -> Vec<T> {
    rows.into_iter()
        .skip(page.offset as usize)
        .take(page.limit as usize)
        .collect()
}
