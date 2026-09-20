//! Use cases around saved scenarios.

use std::sync::Arc;

use uuid::Uuid;
use water_sort_format::Board;

use super::{ApplicationError, Page, ScenarioRepository};
use crate::domain::Scenario;

#[derive(Clone)]
pub struct ScenarioService {
    repository: Arc<dyn ScenarioRepository>,
}

impl ScenarioService {
    pub fn new(repository: Arc<dyn ScenarioRepository>) -> Self {
        Self { repository }
    }

    /// Save a submitted board under a name so it can be replayed later.
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        board: Board,
    ) -> Result<Scenario, ApplicationError> {
        if name.trim().is_empty() {
            return Err(ApplicationError::Invalid("a scenario needs a name".into()));
        }

        let scenario = Scenario::new(name, description, board);
        self.repository.save(&scenario).await?;
        Ok(scenario)
    }

    pub async fn get(&self, id: Uuid) -> Result<Scenario, ApplicationError> {
        self.repository
            .find(id)
            .await?
            .ok_or(ApplicationError::NotFound { resource: "scenario", id })
    }

    pub async fn list(&self, page: Page) -> Result<Vec<Scenario>, ApplicationError> {
        Ok(self.repository.list(page).await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::NotFound { resource: "scenario", id })
        }
    }
}
