# Development

Everything runs from the root, through the `Makefile`. `make help` lists every
target; this page says what is worth knowing beyond the one line summaries.

## What you need

| | Version | Needed for |
| --- | --- | --- |
| Rust | edition 2024 (1.85+) | the backend |
| Node | 22+ | the front end |
| Docker | with `docker compose` | the stack, the images |
| PostgreSQL | 16 | only when running the API on the host with `STORAGE=postgres` |

## The whole stack, in Docker

```sh
make dev        # front end on :5173, API on :8080, PostgreSQL next to them
make logs
make down       # stop; `make clean-volumes` also drops the database
```

## On the host, two terminals

```sh
make api-memory   # the API on :8080, no database, nothing kept
make front        # the front end on :5173, proxying /api to it
```

`make api` uses PostgreSQL instead, at `DATABASE_URL`, which defaults to
`postgres://water_sort:water_sort@localhost:5432/water_sort`. Override it on the
command line: `make api DATABASE_URL=postgres://…`.

The Vite proxy sends `/api` and `/health` to `http://localhost:8080`; point it
somewhere else with `API_PROXY_TARGET`.

## The CLI

```sh
make solve                                        # backend/levels/level145.json
make solve LEVEL=backend/levels/level146.json
```

## Checks

```sh
make check    # the tests of both sides, plus the front end type check
make test     # the tests alone
make fmt      # rustfmt over the workspace
```

`make test-backend` runs the rules, the domain and the HTTP tests against the
in-memory adapters, so it needs nothing installed. The PostgreSQL adapter tests
are skipped unless `TEST_DATABASE_URL` points at a database they may migrate and
write to:

```sh
cd backend && TEST_DATABASE_URL=postgres://… cargo test
```

`make test-frontend` runs vitest over the rules mirrored in TypeScript.

`make check-strict` adds `cargo fmt --check` and `cargo clippy -D warnings`.
Both **fail today**: the original solver sources — `crates/core/src/pipe.rs`,
`game.rs`, `solver.rs` — predate either tool being run on them, and were moved
into the workspace untouched so the algorithm stayed byte for byte the same.
Clearing that backlog is its own change; until then `make check` is what a
contributor and a CI job should run.

## CI

`.github/workflows/ci.yml` runs on every pull request targeting `master` (and
`main`, if the default branch is ever renamed), and on every push to those
branches. There is no `paths:` filter anywhere on purpose: a required check
that gets skipped never reports, and a pull request waiting on a check that
never arrives cannot be merged.

| Job | Runs | Blocking |
| --- | --- | --- |
| backend · tests | `cargo test --workspace --locked` | yes |
| frontend · types and tests | `npm run lint` (`tsc -b`), then `npm test` | yes |
| backend · dependency audit | `cargo audit` | yes |
| frontend · dependency audit | `npm audit --audit-level=high` | yes |
| backend · rustfmt and clippy | `cargo fmt --check`, `cargo clippy -D warnings` | **no** — the backlog above |

Locally, `make check` covers the first two and `make audit` the next two.
`make audit` is kept out of `make check` because it fetches an advisory
database, while `check` stays offline.

### Waived advisories

`cargo audit` reads `backend/.cargo/audit.toml`, which lists the advisories
that are waived and why. Today that is one:

* **RUSTSEC-2023-0071** (`rsa`, Marvin timing sidechannel). `rsa` reaches
  `Cargo.lock` through `sqlx-mysql`, which nothing enables — sea-orm is built
  with `sqlx-postgres` alone, so `cargo tree -i rsa` resolves to nothing and the
  crate is never compiled into any binary. No fixed release exists either. It
  has to be un-waived if this workspace ever enables the MySQL driver.

The job prints that file next to the check, so a waiver stays visible in the log
rather than silently disappearing from the run that enforces it.

`npm audit` blocks at high and critical. The tree currently carries two moderate
advisories in `vitest`, a dev dependency whose only fix is a major version bump;
the job prints every severity in a second, non-blocking step.

## Migrations

They are SeaORM migrations and run at start up by default. To run them as a
separate step instead — which is what you want when several API instances start
at once, since concurrent migrators race on the same tables:

```sh
make migrate
RUN_MIGRATIONS=false make api
```

## Configuration

The API reads its configuration from the environment:

| Variable | Default | What it does |
| --- | --- | --- |
| `STORAGE` | `postgres` | `postgres` or `memory` |
| `DATABASE_URL` | — | required when `STORAGE=postgres` |
| `DATABASE_MAX_CONNECTIONS` | `10` | size of the connection pool |
| `RUN_MIGRATIONS` | `true` | apply the migrations at start up |
| `BIND_ADDRESS` | `0.0.0.0:8080` | where to listen |
| `SOLVER_TIMEOUT_SECONDS` | `30` | a solve answers `504` past this |
| `SOLVER_STACK_SIZE_MB` | `256` | stack of the solver thread |
| `CORS_ALLOWED_ORIGINS` | `*` | origins a browser may call the API from |
| `RUST_LOG` | `water_sort_api=info` | log filter |

The front end reads three, all optional:

| Variable | When | What it does |
| --- | --- | --- |
| `API_PROXY_TARGET` | `make front` | where the Vite proxy forwards `/api` and `/health` |
| `VITE_API_BASE_URL` | build time | absolute API origin, when the app is not served behind a proxy |
| `API_URL` | container start | where nginx forwards `/api` and `/health` |

## Adding an endpoint

1. The use case goes in `backend/crates/api/src/application/`, against a port —
   not against SQL.
2. The handler goes in `backend/crates/api/src/adapters/inbound/http/`, and does
   no game logic: it translates a request into a use case call and back.
3. The payload goes in `dto.rs`, and its mirror in `frontend/src/api/types.ts`.
4. The call goes in `frontend/src/api/`, which is the only place that fetches.
5. Document it in [`api.md`](api.md).
