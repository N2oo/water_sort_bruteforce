use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use uuid::Uuid;
use water_sort_format::Board;

use super::entities::{game_move, game_session};
use super::{backend, corrupted};
use crate::application::{GameSessionRepository, Page, RepositoryError};
use crate::domain::{GameSession, GameStatus, PlayedMove};

pub struct PostgresGameSessionRepository {
    connection: DatabaseConnection,
}

impl PostgresGameSessionRepository {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl GameSessionRepository for PostgresGameSessionRepository {
    /// Store the whole aggregate in one transaction.
    ///
    /// The history is rewritten rather than appended to, because a rollback has
    /// to make the dropped moves disappear from the storage as well.
    async fn save(&self, session: &GameSession) -> Result<(), RepositoryError> {
        let row = game_session::ActiveModel {
            id: Set(session.id),
            scenario_id: Set(session.scenario_id),
            name: Set(session.name.clone()),
            status: Set(session.status.as_str().to_owned()),
            initial_board: Set(serde_json::to_value(&session.initial_board)
                .map_err(|error| corrupted(session.id, error))?),
            current_board: Set(serde_json::to_value(&session.current_board)
                .map_err(|error| corrupted(session.id, error))?),
            created_at: Set(session.created_at.into()),
            updated_at: Set(session.updated_at.into()),
        };

        let mut moves = Vec::with_capacity(session.moves.len());
        for played in &session.moves {
            moves.push(game_move::ActiveModel {
                id: Set(played.id),
                session_id: Set(session.id),
                sequence: Set(played.sequence),
                from_pipe: Set(played.from_pipe),
                to_pipe: Set(played.to_pipe),
                board_after: Set(serde_json::to_value(&played.board_after)
                    .map_err(|error| corrupted(played.id, error))?),
                played_at: Set(played.played_at.into()),
            });
        }

        let session_id = session.id;
        self.connection
            .transaction::<_, (), sea_orm::DbErr>(move |transaction| {
                Box::pin(async move {
                    game_session::Entity::insert(row)
                        .on_conflict(
                            OnConflict::column(game_session::Column::Id)
                                .update_columns([
                                    game_session::Column::ScenarioId,
                                    game_session::Column::Name,
                                    game_session::Column::Status,
                                    game_session::Column::CurrentBoard,
                                    game_session::Column::UpdatedAt,
                                ])
                                .to_owned(),
                        )
                        .exec(transaction)
                        .await?;

                    game_move::Entity::delete_many()
                        .filter(game_move::Column::SessionId.eq(session_id))
                        .exec(transaction)
                        .await?;

                    if !moves.is_empty() {
                        game_move::Entity::insert_many(moves).exec(transaction).await?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|error| RepositoryError::Backend(anyhow::Error::new(error)))?;

        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Option<GameSession>, RepositoryError> {
        let Some(row) = game_session::Entity::find_by_id(id)
            .one(&self.connection)
            .await
            .map_err(backend)?
        else {
            return Ok(None);
        };

        let moves = game_move::Entity::find()
            .filter(game_move::Column::SessionId.eq(id))
            .order_by_asc(game_move::Column::Sequence)
            .all(&self.connection)
            .await
            .map_err(backend)?;

        Ok(Some(into_domain(row, moves)?))
    }

    async fn list(&self, page: Page) -> Result<Vec<GameSession>, RepositoryError> {
        let rows = game_session::Entity::find()
            .order_by_desc(game_session::Column::CreatedAt)
            .order_by_desc(game_session::Column::Id)
            .limit(page.limit)
            .offset(page.offset)
            .all(&self.connection)
            .await
            .map_err(backend)?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let all_moves = game_move::Entity::find()
            .filter(game_move::Column::SessionId.is_in(ids))
            .order_by_asc(game_move::Column::Sequence)
            .all(&self.connection)
            .await
            .map_err(backend)?;

        rows.into_iter()
            .map(|row| {
                let moves = all_moves
                    .iter()
                    .filter(|played| played.session_id == row.id)
                    .cloned()
                    .collect();
                into_domain(row, moves)
            })
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let outcome = game_session::Entity::delete_many()
            .filter(game_session::Column::Id.eq(id))
            .exec(&self.connection)
            .await
            .map_err(backend)?;

        Ok(outcome.rows_affected > 0)
    }
}

fn into_domain(
    row: game_session::Model,
    moves: Vec<game_move::Model>,
) -> Result<GameSession, RepositoryError> {
    let initial_board: Board =
        serde_json::from_value(row.initial_board).map_err(|error| corrupted(row.id, error))?;
    let current_board: Board =
        serde_json::from_value(row.current_board).map_err(|error| corrupted(row.id, error))?;
    let status = GameStatus::parse(&row.status).ok_or_else(|| {
        corrupted(row.id, format!("unknown game status '{}'", row.status))
    })?;

    let moves = moves
        .into_iter()
        .map(|played| {
            let board_after: Board = serde_json::from_value(played.board_after)
                .map_err(|error| corrupted(played.id, error))?;
            Ok(PlayedMove {
                id: played.id,
                sequence: played.sequence,
                from_pipe: played.from_pipe,
                to_pipe: played.to_pipe,
                board_after,
                played_at: played.played_at.into(),
            })
        })
        .collect::<Result<Vec<_>, RepositoryError>>()?;

    Ok(GameSession {
        id: row.id,
        scenario_id: row.scenario_id,
        name: row.name,
        status,
        initial_board,
        current_board,
        moves,
        created_at: row.created_at.into(),
        updated_at: row.updated_at.into(),
    })
}
