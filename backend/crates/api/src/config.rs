//! Configuration, read from the environment.

use std::time::Duration;

use anyhow::{Context, bail};

/// Where the state lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Storage {
    /// PostgreSQL, through SeaORM. The default, and the only durable one.
    Postgres { url: String, max_connections: u32, run_migrations: bool },
    /// Everything in the process memory: handy to try the API out, lost on exit.
    Memory,
}

/// Which origins a browser may call the API from.
///
/// The API carries no credentials and no session cookie, so `Any` — the
/// default — only means "a page served from anywhere may call it", which is
/// what the React front end needs when it is served from its own port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorsOrigins {
    Any,
    List(Vec<String>),
}

impl CorsOrigins {
    /// `CORS_ALLOWED_ORIGINS=http://localhost:5173,https://example.org`, or
    /// `*` (the default) for any origin.
    fn from_env() -> Self {
        match std::env::var("CORS_ALLOWED_ORIGINS") {
            Ok(raw) if raw.trim() != "*" && !raw.trim().is_empty() => Self::List(
                raw.split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(str::to_string)
                    .collect(),
            ),
            _ => Self::Any,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub storage: Storage,
    pub solver_timeout: Duration,
    pub solver_stack_size: usize,
    pub cors_origins: CorsOrigins,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let storage = match std::env::var("STORAGE").unwrap_or_else(|_| "postgres".into()).as_str() {
            "memory" => Storage::Memory,
            "postgres" => {
                let url = std::env::var("DATABASE_URL").context(
                    "DATABASE_URL is required (or set STORAGE=memory for a throw away run)",
                )?;
                Storage::Postgres {
                    url,
                    max_connections: parse("DATABASE_MAX_CONNECTIONS", 10)?,
                    run_migrations: parse("RUN_MIGRATIONS", true)?,
                }
            }
            other => bail!("unknown STORAGE '{other}', expected 'postgres' or 'memory'"),
        };

        Ok(Self {
            bind_address: std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            storage,
            solver_timeout: Duration::from_secs(parse("SOLVER_TIMEOUT_SECONDS", 30)?),
            solver_stack_size: parse::<usize>("SOLVER_STACK_SIZE_MB", 256)? * 1024 * 1024,
            cors_origins: CorsOrigins::from_env(),
        })
    }
}

fn parse<T>(variable: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(variable) {
        Ok(raw) => raw
            .parse()
            .map_err(|error| anyhow::anyhow!("invalid {variable}='{raw}': {error}")),
        Err(_) => Ok(default),
    }
}
