# The HTTP API

Base URL: `http://localhost:8080` when running it by hand, and the same paths
through the front end, which proxies `/api` and `/health` to it.

## Endpoints

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

## A board

Two shapes are accepted, so the historical level files keep working:

```json
{ "pipes": { "P1": ["Brown", "Lemon", "Blue", "Grey"], "P13": [] } }
{ "pipes": [ { "label": "P1", "colors": ["Brown"] }, { "label": "P2", "colors": [] } ] }
```

Colors are `Grey` (or `Gray`), `Blue`, `Lemon`, `Brown`, `Green`, `Red`,
`LightGreen`, `LightBlue`, `Pink`, `Orange`, `Purple`, `Yellow`, parsed case
insensitively. A pipe holds at most four units, listed **bottom first**.

Everything the API hands out is identified by a UUID: scenarios, games, moves,
and the pipes themselves. A pipe keeps a human readable `label` (`P1`, `P2`, …)
next to its id so a board stays readable.

## A walk through

Save a puzzle, then start a game on it:

```sh
curl -sX POST localhost:8080/api/v1/scenarios \
  -H 'content-type: application/json' \
  -d "{\"name\": \"Level 145\", \"puzzle\": $(cat backend/levels/level145.json)}"

curl -sX POST localhost:8080/api/v1/games \
  -H 'content-type: application/json' \
  -d '{"scenario_id": "<scenario uuid>", "name": "first try"}'
```

Or submit the board directly, optionally saving it on the way:

```sh
curl -sX POST localhost:8080/api/v1/games \
  -H 'content-type: application/json' \
  -d "{\"puzzle\": $(cat backend/levels/level146.json), \"save_as_scenario\": \"Level 146\"}"
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
  -d "{\"puzzle\": $(cat backend/levels/level145.json)}"
```

```json
{
  "solved": true,
  "moves_count": 50,
  "moves": [{ "from": "4a1c…", "to": "9b0d…", "from_label": "P10", "to_label": "P13", "notation": "P10->P13" }]
}
```

## Errors

Errors always come back in the same envelope:

```json
{ "error": { "code": "illegal_move", "message": "pouring '…' into '…' is not allowed by the rules" } }
```

`illegal_move`, `unknown_pipe`, `same_pipe` and `invalid_board` answer `422`,
`already_solved`, `not_enough_moves` and `nothing_to_roll_back` answer `409`,
`solver_timeout` answers `504`.
