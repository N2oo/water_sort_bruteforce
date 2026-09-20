use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use uuid::Uuid;
use water_sort_format::Board;

use super::entities::scenario;
use super::{backend, corrupted};
use crate::application::{Page, RepositoryError, ScenarioRepository};
use crate::domain::Scenario;

pub struct PostgresScenarioRepository {
    connection: DatabaseConnection,
}

impl PostgresScenarioRepository {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl ScenarioRepository for PostgresScenarioRepository {
    async fn save(&self, scenario: &Scenario) -> Result<(), RepositoryError> {
        let row = scenario::ActiveModel {
            id: Set(scenario.id),
            name: Set(scenario.name.clone()),
            description: Set(scenario.description.clone()),
            board: Set(serde_json::to_value(&scenario.board)
                .map_err(|error| corrupted(scenario.id, error))?),
            created_at: Set(scenario.created_at.into()),
        };

        scenario::Entity::insert(row)
            .on_conflict(
                OnConflict::column(scenario::Column::Id)
                    .update_columns([
                        scenario::Column::Name,
                        scenario::Column::Description,
                        scenario::Column::Board,
                    ])
                    .to_owned(),
            )
            .exec(&self.connection)
            .await
            .map_err(backend)?;

        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Option<Scenario>, RepositoryError> {
        scenario::Entity::find_by_id(id)
            .one(&self.connection)
            .await
            .map_err(backend)?
            .map(into_domain)
            .transpose()
    }

    async fn list(&self, page: Page) -> Result<Vec<Scenario>, RepositoryError> {
        scenario::Entity::find()
            .order_by_desc(scenario::Column::CreatedAt)
            .order_by_desc(scenario::Column::Id)
            .limit(page.limit)
            .offset(page.offset)
            .all(&self.connection)
            .await
            .map_err(backend)?
            .into_iter()
            .map(into_domain)
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let outcome = scenario::Entity::delete_many()
            .filter(scenario::Column::Id.eq(id))
            .exec(&self.connection)
            .await
            .map_err(backend)?;

        Ok(outcome.rows_affected > 0)
    }
}

fn into_domain(row: scenario::Model) -> Result<Scenario, RepositoryError> {
    let board: Board =
        serde_json::from_value(row.board).map_err(|error| corrupted(row.id, error))?;

    Ok(Scenario {
        id: row.id,
        name: row.name,
        description: row.description,
        board,
        created_at: row.created_at.into(),
    })
}
