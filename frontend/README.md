# Water sort — front end

A React front end for `water-sort-api`. It follows one path:

```
        design or paste a scenario
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
     play it            solve it
 (a game session)   (a solution, replayable)
```

* **Designer** (`/design`) — build a board by hand, generate a random solvable
  one, or paste a level file. It is validated against the same rules the API
  enforces, and shown as the exact JSON payload every endpoint accepts.
* **Solver** (`/solve`) — ask for a solution for the board being designed, for a
  saved scenario, or for JSON pasted straight in. The answer is replayed step by
  step on the board.
* **Games** (`/games`, `/games/{id}`) — play a session: pour by clicking two
  pipes, undo, roll back several moves, reset, solve from where you stand and
  have the whole solution played back for you.
* **Scenarios** (`/scenarios`) — the saved boards: play, solve, edit a copy, drop.

## Running it

```sh
npm install
npm run dev            # http://localhost:5173
```

The dev server proxies `/api` and `/health` to `http://localhost:8080`, so start
the API next to it:

```sh
STORAGE=memory cargo run -p water-sort-api
```

| Variable | Where | Default | What it does |
| --- | --- | --- | --- |
| `API_PROXY_TARGET` | dev server | `http://localhost:8080` | where the proxy sends `/api` and `/health` |
| `VITE_API_BASE_URL` | build | empty | absolute API origin, when the app is *not* served behind a proxy |
| `API_URL` | Docker image | `http://api:8080` | what nginx forwards `/api` and `/health` to |

Leaving `VITE_API_BASE_URL` empty is the usual case: the app then calls its own
origin, which the dev proxy or nginx forwards. Set it when the front end is
served from somewhere that cannot proxy — the API then needs the calling origin
in `CORS_ALLOWED_ORIGINS` (it allows any origin by default).

```sh
npm run build          # type check, then bundle into dist/
npm run preview        # serve dist/ locally
npm test               # the rules, the generator and the JSON import
```

## How it is put together

```
src/
  api/        types, the fetch client, and the TanStack Query hooks
  game/       the rules mirrored in TypeScript, the palette, the draft model
  components/ the board, a pipe, the solution stepper, the JSON panels
  pages/      designer, solver, games, scenarios
  state/      the scenario being designed, kept in localStorage
```

**The API decides.** Every move is posted to `POST /games/{id}/moves` and the
board that comes back is the one drawn; the client never rules on a pour. The
copy of the rules in `src/game/rules.ts` serves the things that cannot wait for
a round trip — highlighting the pipes a selected pipe may pour into, replaying a
solution locally, warning about a board the API would refuse. It mirrors
`crates/core/src/pipe.rs` and `src/game/rules.test.ts` pins that down.

**Every read is a query, every write a mutation.** `src/api/queries.ts` is the
only place that knows the cache: a mutation that changes a game seeds
`['games', id]` with the session the API just answered with and invalidates the
history and the listings. Solving is a mutation too — it is a long request whose
answer belongs to the click that asked for it, not to a cache key.

**Errors keep their code.** The API answers `{ "error": { "code", "message" } }`;
`ApiError` carries both, so `illegal_move` and `solver_timeout` can be told
apart from a network failure without parsing sentences.
