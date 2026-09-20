//! Use cases around running games.

use std::sync::Arc;

use uuid::Uuid;
use water_sort_format::Board;

use super::{ApplicationError, GameSessionRepository, Page, ScenarioRepository};
use crate::domain::{GameSession, PlayedMove};

/// How a game is started: from a scenario that was saved earlier, or from a
/// board submitted on the spot.
#[derive(Debug, Clone)]
pub enum StartGame {
    FromScenario { scenario_id: Uuid, name: Option<String> },
    FromBoard { board: Board, name: Option<String>, save_as_scenario: Option<String> },
}

#[derive(Clone)]
pub struct GameService {
    sessions: Arc<dyn GameSessionRepository>,
    scenarios: Arc<dyn ScenarioRepository>,
}

impl GameService {
    pub fn new(
        sessions: Arc<dyn GameSessionRepository>,
        scenarios: Arc<dyn ScenarioRepository>,
    ) -> Self {
        Self { sessions, scenarios }
    }

    pub async fn start(&self, request: StartGame) -> Result<GameSession, ApplicationError> {
        let session = match request {
            StartGame::FromScenario { scenario_id, name } => {
                let scenario = self
                    .scenarios
                    .find(scenario_id)
                    .await?
                    .ok_or(ApplicationError::NotFound { resource: "scenario", id: scenario_id })?;
                GameSession::start(scenario.board, Some(scenario.id), name)?
            }
            StartGame::FromBoard { board, name, save_as_scenario } => {
                let scenario_id = match save_as_scenario {
                    Some(scenario_name) => {
                        let scenario = crate::domain::Scenario::new(
                            scenario_name,
                            None,
                            board.clone(),
                        );
                        self.scenarios.save(&scenario).await?;
                        Some(scenario.id)
                    }
                    None => None,
                };
                GameSession::start(board, scenario_id, name)?
            }
        };

        self.sessions.save(&session).await?;
        Ok(session)
    }

    pub async fn get(&self, id: Uuid) -> Result<GameSession, ApplicationError> {
        self.sessions
            .find(id)
            .await?
            .ok_or(ApplicationError::NotFound { resource: "game", id })
    }

    pub async fn list(&self, page: Page) -> Result<Vec<GameSession>, ApplicationError> {
        Ok(self.sessions.list(page).await?)
    }

    /// Play one pour on a running game.
    pub async fn play(
        &self,
        id: Uuid,
        from: Uuid,
        to: Uuid,
    ) -> Result<(GameSession, PlayedMove), ApplicationError> {
        let mut session = self.get(id).await?;
        let played = session.play(from, to)?.clone();
        self.sessions.save(&session).await?;
        Ok((session, played))
    }

    /// Play several pours in a row; the game is stored once, and nothing is
    /// stored at all if one of them is refused.
    pub async fn play_all(
        &self,
        id: Uuid,
        moves: &[(Uuid, Uuid)],
    ) -> Result<(GameSession, Vec<PlayedMove>), ApplicationError> {
        let mut session = self.get(id).await?;
        let mut played = Vec::with_capacity(moves.len());
        for (from, to) in moves {
            played.push(session.play(*from, *to)?.clone());
        }
        self.sessions.save(&session).await?;
        Ok((session, played))
    }

    /// Undo the `steps` last moves; the rolled back moves are dropped for good.
    pub async fn roll_back(
        &self,
        id: Uuid,
        steps: usize,
    ) -> Result<(GameSession, Vec<PlayedMove>), ApplicationError> {
        let mut session = self.get(id).await?;
        let dropped = session.roll_back(steps)?;
        self.sessions.save(&session).await?;
        Ok((session, dropped))
    }

    /// Undo everything, back to the board the game started from.
    pub async fn reset(
        &self,
        id: Uuid,
    ) -> Result<(GameSession, Vec<PlayedMove>), ApplicationError> {
        let mut session = self.get(id).await?;
        let dropped = session.reset()?;
        self.sessions.save(&session).await?;
        Ok((session, dropped))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        if self.sessions.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::NotFound { resource: "game", id })
        }
    }
}
