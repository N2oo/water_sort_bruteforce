# Water sort

A water sort puzzle solver, packaged as a small monorepo: the original
bruteforce solver is still a CLI, and the same rules now also back an HTTP API
that creates game sessions, plays moves on them, rolls them back and solves
them.

```
crates/
  core/       the game rules and the bruteforce solver — pure, no I/O
  format/     the JSON shape of a board, shared by the CLI and the API
  cli/        driving adapter: the command line solver
  api/        driving adapter: the HTTP API, plus its driven adapters
  migration/  the PostgreSQL schema (SeaORM migrations)
levels/       the historical level files
```

## Architecture

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

## The CLI

```sh
cargo run -p water-sort-cli -- levels/level145.json
cargo run -p water-sort-cli -- levels/level145.json --json
```

```
✓ Solution trouvée en 50 mouvements !

  Mouvement 1 : P10->P13
  Mouvement 2 : P10->P14
  ...
```

It accepts the historical level files unchanged:

```json
{ "pipes": { "P1": ["Brown", "Lemon", "Blue", "Grey"], "P13": [] } }
```

## The API

### Running it

```sh
# with PostgreSQL (the default, and the only durable storage)
DATABASE_URL=postgres://water_sort:water_sort@localhost/water_sort \
  cargo run -p water-sort-api

# or without a database at all, to try things out
STORAGE=memory cargo run -p water-sort-api
```

| Variable | Default | What it does |
| --- | --- | --- |
| `STORAGE` | `postgres` | `postgres` or `memory` |
| `DATABASE_URL` | — | required when `STORAGE=postgres` |
| `DATABASE_MAX_CONNECTIONS` | `10` | size of the connection pool |
| `RUN_MIGRATIONS` | `true` | apply the migrations at start up |
| `BIND_ADDRESS` | `0.0.0.0:8080` | where to listen |
| `SOLVER_TIMEOUT_SECONDS` | `30` | a solve answers `504` past this |
| `SOLVER_STACK_SIZE_MB` | `256` | stack of the solver thread |
| `RUST_LOG` | `water_sort_api=info` | log filter |

### Endpoints

| Method | Path | What it does |
| --- | --- | --- |
| `GET` | `/health` | liveness |
| `GET` | `/` | the list below, as JSON |
| `POST` | `/api/v1/puzzles/solve` | solve a submitted puzzle, storing nothing |
| `GET` `POST` | `/api/v1/scenarios` | list / save a puzzle under a name |
| `GET` `DELETE` | `/api/v1/scenarios/{id}` | read / drop a saved puzzle |
| `POST` | `/api/v1/scenarios/{id}/solve` | solve a saved puzzle |
| `GET` `POST` | `/api/v1/games` | list / start a game session |
| `GET` `DELETE` | `/api/v1/games/{id}` | read / drop a game session |
| `GET` | `/api/v1/games/{id}/moves` | the history, board snapshots included |
| `POST` | `/api/v1/games/{id}/moves` | play one move, or a batch |
| `POST` | `/api/v1/games/{id}/rollback` | undo the last moves and **drop** them |
| `POST` | `/api/v1/games/{id}/reset` | undo every move |
| `POST` | `/api/v1/games/{id}/solve` | solve the game from where it stands |

Errors always come back in the same envelope:

```json
{ "error": { "code": "illegal_move", "message": "pouring '…' into '…' is not allowed by the rules" } }
```

`illegal_move`, `unknown_pipe`, `same_pipe` and `invalid_board` answer `422`,
`already_solved`, `not_enough_moves` and `nothing_to_roll_back` answer `409`,
`solver_timeout` answers `504`.

### A walk through

Save a puzzle, then start a game on it:

```sh
curl -sX POST localhost:8080/api/v1/scenarios \
  -H 'content-type: application/json' \
  -d "{\"name\": \"Level 145\", \"puzzle\": $(cat levels/level145.json)}"

curl -sX POST localhost:8080/api/v1/games \
  -H 'content-type: application/json' \
  -d '{"scenario_id": "<scenario uuid>", "name": "first try"}'
```

Or submit the board directly, optionally saving it on the way:

```sh
curl -sX POST localhost:8080/api/v1/games \
  -H 'content-type: application/json' \
  -d "{\"puzzle\": $(cat levels/level146.json), \"save_as_scenario\": \"Level 146\"}"
```

A game answers with its pipes, each carrying its own uuid:

```json
{
  "id": "0f2e…",
  "status": "running",
  "moves_played": 0,
  "completed_pipes": 2,
  "pipes": [{ "id": "4a1c…", "label": "P1", "colors": ["Brown", "Lemon", "Blue", "Grey"] }],
  "moves": []
}
```

Play a move — or several in one call, which are all refused together if one of
them is illegal:

```sh
curl -sX POST localhost:8080/api/v1/games/<game>/moves \
  -H 'content-type: application/json' \
  -d '{"from": "<pipe uuid>", "to": "<pipe uuid>"}'

curl -sX POST localhost:8080/api/v1/games/<game>/moves \
  -H 'content-type: application/json' \
  -d '{"moves": [{"from": "…", "to": "…"}, {"from": "…", "to": "…"}]}'
```

Roll back the last two moves. They are **dropped**: they leave the history and
the database, and the next move played takes their place in the numbering.

```sh
curl -sX POST localhost:8080/api/v1/games/<game>/rollback \
  -H 'content-type: application/json' -d '{"steps": 2}'

curl -sX POST localhost:8080/api/v1/games/<game>/rollback   # undoes the last one
curl -sX POST localhost:8080/api/v1/games/<game>/reset      # undoes everything
```

Ask for a solution — of a running game (from its current board, so the moves can
be posted straight back to `/moves`), of a saved scenario, or of a board that is
never stored:

```sh
curl -sX POST localhost:8080/api/v1/games/<game>/solve
curl -sX POST localhost:8080/api/v1/scenarios/<scenario>/solve
curl -sX POST localhost:8080/api/v1/puzzles/solve \
  -H 'content-type: application/json' \
  -d "{\"puzzle\": $(cat levels/level145.json)}"
```

```json
{
  "solved": true,
  "moves_count": 50,
  "moves": [{ "from": "4a1c…", "to": "9b0d…", "from_label": "P10", "to_label": "P13", "notation": "P10->P13" }]
}
```

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

Migrations are SeaORM migrations and run at start up by default. To run them as
a separate step instead:

```sh
RUN_MIGRATIONS=false cargo run -p water-sort-api
DATABASE_URL=postgres://… cargo run -p water-sort-migration -- up
```

Prefer that when several API instances start at once: concurrent migrators race
on the same tables.

## Docker

```sh
docker compose up --build       # API on :8080, PostgreSQL next to it
```

Or the image alone — it carries the API, the migration tool and the CLI:

```sh
docker build -t water-sort .
docker run --rm -p 8080:8080 -e DATABASE_URL=postgres://… water-sort
docker run --rm -e STORAGE=memory -p 8080:8080 water-sort
docker run --rm water-sort water-sort /opt/water-sort/levels/level145.json
```

## Tests

```sh
cargo test                                   # rules, domain, HTTP API
TEST_DATABASE_URL=postgres://…  cargo test   # …and the PostgreSQL adapter
```

The HTTP tests run the real router against the in-memory adapters, so they need
nothing installed. The storage tests are skipped unless `TEST_DATABASE_URL`
points at a database they may migrate and write to.

## Notes

* The solver is the original exhaustive depth first search. It is exponential:
  it answers in milliseconds on the shipped levels, but a hostile board can run
  for a long time, which is what `SOLVER_TIMEOUT_SECONDS` is for. A timed out
  search is abandoned by the request — the thread it runs on has no cancellation
  point and finishes on its own.
* The search explores pipes in identifier order, so the generated pipe uuids are
  handed out in the order the board was submitted in. Solving the same board
  twice therefore gives the same answer, even though the uuids differ.
