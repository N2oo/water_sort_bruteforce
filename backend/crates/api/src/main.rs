//! Composition root: read the configuration, plug the adapters, serve.

use tracing_subscriber::{EnvFilter, fmt};
use water_sort_api::adapters::inbound::http::{cors, router, serve};
use water_sort_api::build_state;
use water_sort_api::config::{Config, Storage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("water_sort_api=info,tower_http=info")),
        )
        .init();

    let config = Config::from_env()?;
    let state = build_state(&config).await?;
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;

    tracing::info!(
        address = %listener.local_addr()?,
        storage = match config.storage {
            Storage::Postgres { .. } => "postgres",
            Storage::Memory => "memory",
        },
        solver_timeout_s = config.solver_timeout.as_secs(),
        "water sort api is listening"
    );

    serve(listener, router(state).layer(cors(&config.cors_origins))).await?;
    Ok(())
}
