# Water sort

A water sort puzzle solver, as a monorepo: the original bruteforce solver is
still a CLI, the same rules back an HTTP API that creates game sessions, plays
moves on them, rolls them back and solves them, and a React front end drives all
of it from a browser.

```
backend/     Rust workspace: game rules, solver, CLI, HTTP API, migrations
frontend/    React front end: Vite, TanStack Query, shadcn/ui
docs/        architecture, the HTTP contract, how to work on it
Makefile     every task, from here
docker-compose.yml   PostgreSQL + API + front end
```

Each side owns its dependencies, its build and its Dockerfile —
[`backend/Dockerfile`](backend/Dockerfile) and
[`frontend/Dockerfile`](frontend/Dockerfile). Nothing at the root builds
anything by itself.

## Getting started

```sh
make dev
```

The front end comes up on <http://localhost:5173>, the API on
<http://localhost:8080>, with PostgreSQL next to them. Design a board on
`/design`, play it, and ask for a solution when it gets stuck.

Without Docker, two terminals:

```sh
make api-memory   # the API on :8080, no database at all
make front        # the front end on :5173, proxying to it
```

And the original CLI, unchanged:

```sh
make solve LEVEL=backend/levels/level145.json
```

```
✓ Solution trouvée en 50 mouvements !

  Mouvement 1 : P10->P13
  Mouvement 2 : P10->P14
  ...
```

## Every task

`make help` prints them all. The ones you want first:

| Target | What it does |
| --- | --- |
| `make dev` | the whole stack in Docker |
| `make down` / `make logs` | stop it / follow it |
| `make api` `make api-memory` `make front` | run one side on the host |
| `make solve LEVEL=…` | solve a level file with the CLI |
| `make check` | the tests of both sides, plus the front end types |
| `make test` `make lint` `make fmt` | one step of it |
| `make build` | release binaries and a production bundle |

## Where to read next

| | |
| --- | --- |
| [`docs/architecture.md`](docs/architecture.md) | the layout, the hexagon, where the rules live |
| [`docs/api.md`](docs/api.md) | endpoints, payloads, error codes |
| [`docs/development.md`](docs/development.md) | environment variables, migrations, tests |
| [`backend/README.md`](backend/README.md) | the Rust side on its own |
| [`frontend/README.md`](frontend/README.md) | the front end on its own |

## Notes

* The solver is the original exhaustive depth first search. It is exponential:
  it answers in milliseconds on the shipped levels, but a hostile board can run
  for a long time, which is what `SOLVER_TIMEOUT_SECONDS` is for. A timed out
  search is abandoned by the request — the thread it runs on has no cancellation
  point and finishes on its own.
* The search explores pipes in identifier order, so the generated pipe uuids are
  handed out in the order the board was submitted in. Solving the same board
  twice therefore gives the same answer, even though the uuids differ.
