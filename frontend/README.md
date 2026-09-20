# Water sort — front end

A React front end for `water-sort-api`, built on [shadcn/ui](https://ui.shadcn.com).
It follows one path:

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

## shadcn/ui

Everything on screen is a shadcn/ui component. They live in
`src/components/ui`, where the CLI puts them, and `components.json` is set up
so `npx shadcn@latest add <component>` works as usual.

`ui.shadcn.com` is unreachable from some networks, so `npm run ui:add` is the
same thing over a route that stays open: it pulls the sources from the upstream
repository (`shadcn-ui/ui`, style `new-york-v4`) and applies the one rewrite the
CLI applies, turning the monorepo's registry paths into this project's `@/`
alias. `cn` and `radix-ui` are published packages, so those imports are left
alone and the files land **verbatim**.

```sh
npm run ui:add                 # refresh the components already vendored
npm run ui:add dialog popover  # add new ones
```

Nothing in `src/components/ui` is edited by hand. Three files sit next to that
directory and are compositions, not components — a `Card` with shadcn parts
inside, no styling of their own:

| File | What it holds |
| --- | --- |
| `board.tsx` | the board and its pipes, the one thing shadcn has no component for: a `Button` and a `Tooltip` around four Tailwind-drawn slots |
| `json-card.tsx` | a payload in a `ScrollArea`, with copy / download / curl `Button`s |
| `solution-card.tsx` | the solution stepper: a `Slider`, a `ButtonGroup` and a `Badge` per move |

### The theme

`src/index.css` is the only stylesheet, and it only carries a theme: Tailwind,
`shadcn/tailwind.css`, the upstream `new-york-v4` light and dark palettes, and
what this app adds on top —

* `--pipe-*`, the twelve colors of the game. They are the puzzle's colors, not
  the interface's, so they are the same in both themes; `src/game/colors.ts`
  maps a color name onto them.
* `--success` and `--attention`, the two states a board has that a neutral theme
  has no token for: a legal destination or a settled pipe, and the pipe a pour
  starts from.

Dark is the default; the switch in the header is `next-themes`, which the
vendored `sonner.tsx` already depends on.

## How it is put together

```
src/
  api/        types, the fetch client, and the TanStack Query hooks
  game/       the rules mirrored in TypeScript, the palette, the draft model
  components/ ui/ (shadcn) + the three compositions above
  lib/        cn, and the API error → sentence helper
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
apart from a network failure without parsing sentences. They surface as a
`sonner` toast, or as an `Alert` when they belong to the board on screen.

### One known warning

In **development**, React logs `Encountered a script tag while rendering React
component` twice on boot. It comes from `next-themes`, which renders its
no-flash script inside the provider; React 19 warns about that on the client.
It is a dev-only check — the production build is clean — and the alternative
would be hand-rolling a theme provider, which this front end deliberately does
not do.
