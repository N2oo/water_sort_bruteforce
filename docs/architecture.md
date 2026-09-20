# Architecture

## The repository

```
backend/     the Rust workspace: game rules, solver, CLI, HTTP API, migrations
frontend/    the React front end (Vite, TanStack Query, shadcn/ui)
docs/        what you are reading
Makefile     every task, from the root
docker-compose.yml   PostgreSQL + API + front end
```

Each side owns its build, its dependencies and its image:

| | backend | frontend |
| --- | --- | --- |
| Toolchain | cargo | npm |
| Manifest | `backend/Cargo.toml` | `frontend/package.json` |
| Image | `backend/Dockerfile` | `frontend/Dockerfile` |
| Ignored by Docker | `backend/.dockerignore` | `frontend/.dockerignore` |

Nothing at the root builds anything by itself. The `Makefile` delegates, and
`docker-compose.yml` points at the two build contexts. Adding a third side —
a mobile client, a worker — means a folder, a Dockerfile and a few targets, and
nothing else moves.

## The backend

```
backend/
  crates/
    core/       the game rules and the bruteforce solver — pure, no I/O
    format/     the JSON shape of a board, shared by the CLI and the API
    cli/        driving adapter: the command line solver
    api/        driving adapter: the HTTP API, plus its driven adapters
    migration/  the PostgreSQL schema (SeaORM migrations)
  levels/       the historical level files
```

The API is a hexagon. Nothing inside it knows about HTTP or SQL.

```
                    driving adapter                 ports                    driven adapters
  HTTP client ──▶ adapters::inbound::http ──▶ application::{ScenarioService,  ──▶ PostgresScenarioRepository
                                              GameService, SolvingService}        PostgresGameSessionRepository
                                                      │                           InMemory*Repository (tests)
                                                      └──▶ PuzzleSolver ──────▶ BruteforceSolver (water-sort-core)
```

| Port | Declared in | Adapters |
| --- | --- | --- |
| `ScenarioRepository` | `application::ports` | `PostgresScenarioRepository`, `InMemoryScenarioRepository` |
| `GameSessionRepository` | `application::ports` | `PostgresGameSessionRepository`, `InMemoryGameSessionRepository` |
| `PuzzleSolver` | `application::ports` | `BruteforceSolver` |

`crates/core/src/pipe.rs`, `crates/core/src/game.rs` and the search in
`crates/core/src/solver.rs` are the original implementation, moved into a
library crate **without a change to the rules or to the algorithm**. Every
board transition in the API goes through them: `Puzzle::play` asks
`Pipe::can_pour` before every single unit it moves, so there is exactly one
definition of what a legal pour is.

Everything the API hands out is identified by a UUID: scenarios, games, moves,
and the pipes themselves. A pipe keeps a human readable `label` (`P1`, `P2`, …)
next to its id so a board stays readable.

## The front end

The front end is a fourth adapter driving the same hexagon, over HTTP. One page
per step of the path a scenario takes:

| Route | What it does |
| --- | --- |
| `/design` | build a board, generate a random solvable one, or paste a level file |
| `/solve` | solve the board being designed, a saved scenario, or pasted JSON |
| `/games/{id}` | play a session: pour, undo, roll back, reset, solve from here |
| `/scenarios` | the saved boards: play, solve, edit a copy, drop |

The API decides every move: the client posts to `/games/{id}/moves` and draws
the board that comes back. The copy of the rules in `frontend/src/game/rules.ts`
only serves what cannot wait for a round trip — the legal targets of a selected
pipe, a local replay of a solution, and the warnings the designer shows before
a board is submitted. [`../frontend/README.md`](../frontend/README.md) has the
rest.

## One origin, usually

The browser normally only talks to the host it was served from:

| | who serves the app | who forwards `/api` |
| --- | --- | --- |
| development | Vite on `:5173` | the Vite proxy, to `API_PROXY_TARGET` |
| image | nginx on `:80` | nginx, to `API_URL` |

Set `VITE_API_BASE_URL` at build time to call an API on another origin instead —
that origin then has to be in the API's `CORS_ALLOWED_ORIGINS`, which allows any
origin by default.

## Storage

State lives in PostgreSQL. Boards are stored as `jsonb`, which keeps the schema
stable whatever a board looks like.

| Table | Holds |
| --- | --- |
| `scenarios` | saved puzzles |
| `game_sessions` | a game, its initial board and its current board |
| `game_moves` | one row per played move, with the board it produced |

Storing the resulting board on every move is what makes a rollback cheap and
exact: dropping the tail of the history restores the board it points at, and the
dropped rows are deleted in the same transaction that rewrites the game.
