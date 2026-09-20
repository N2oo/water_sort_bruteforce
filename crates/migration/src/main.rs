//! `water-sort-migration up`, `down`, `status`, ... See `sea-orm-cli` for the
//! full list of accepted sub commands.

use sea_orm_migration::prelude::*;
use water_sort_migration::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
