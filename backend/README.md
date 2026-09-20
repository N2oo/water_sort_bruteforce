# Backend

The Rust side: the game rules, the bruteforce solver, the CLI that has always
driven it, and the HTTP API that now does too.

```
crates/
  core/       the game rules and the bruteforce solver — pure, no I/O
  format/     the JSON shape of a board, shared by the CLI and the API
  cli/        driving adapter: the command line solver
  api/        driving adapter: the HTTP API, plus its driven adapters
  migration/  the PostgreSQL schema (SeaORM migrations)
levels/       the historical level files
Dockerfile    one image carrying the API, the migration tool and the CLI
```

Everything here runs from this directory, or through the `Makefile` at the root
of the repository, which is the shorter way (`make api`, `make test-backend`,
`make solve`).

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

```sh
# with PostgreSQL (the default, and the only durable storage)
DATABASE_URL=postgres://water_sort:water_sort@localhost/water_sort \
  cargo run -p water-sort-api

# or without a database at all, to try things out
STORAGE=memory cargo run -p water-sort-api
```

Its endpoints, payloads and error codes are in [`../docs/api.md`](../docs/api.md),
its configuration in [`../docs/development.md`](../docs/development.md), and the
shape of the hexagon in [`../docs/architecture.md`](../docs/architecture.md).

## Tests

```sh
cargo test                                   # rules, domain, HTTP API
TEST_DATABASE_URL=postgres://…  cargo test   # …and the PostgreSQL adapter
```

The HTTP tests run the real router against the in-memory adapters, so they need
nothing installed. The storage tests are skipped unless `TEST_DATABASE_URL`
points at a database they may migrate and write to.

## Migrations

SeaORM migrations, run at start up by default. As a separate step instead —
which is what you want when several API instances start at once, since
concurrent migrators race on the same tables:

```sh
RUN_MIGRATIONS=false cargo run -p water-sort-api
DATABASE_URL=postgres://… cargo run -p water-sort-migration -- up
```

## The image

Built from this directory, and carrying all three binaries:

```sh
docker build -t water-sort-api .
docker run --rm -p 8080:8080 -e DATABASE_URL=postgres://… water-sort-api
docker run --rm -e STORAGE=memory -p 8080:8080 water-sort-api
docker run --rm water-sort-api water-sort /opt/water-sort/levels/level145.json
```
