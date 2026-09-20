use chrono::{DateTime, Utc};
use uuid::Uuid;
use water_sort_format::Board;

/// A board saved under a name, ready to be replayed as many times as wanted.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub board: Board,
    pub created_at: DateTime<Utc>,
}

impl Scenario {
    pub fn new(name: String, description: Option<String>, board: Board) -> Self {
        Self { id: Uuid::new_v4(), name, description, board, created_at: Utc::now() }
    }
}
