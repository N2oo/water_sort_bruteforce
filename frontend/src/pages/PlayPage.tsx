/**
 * Play a game session.
 *
 * Every move goes to the API and the board that comes back is the one drawn —
 * the client never decides the outcome of a pour. The local rules are used for
 * one thing only: lighting up the pipes a selected pipe may pour into, so the
 * board answers the pointer without a round trip.
 */

import { useEffect, useMemo, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'

import {
  useGame,
  useGameMoves,
  usePlayMoves,
  useResetGame,
  useRollBack,
  useSolveGame,
} from '../api/queries'
import { BoardView } from '../components/BoardView'
import { EmptyState, ErrorNotice, Spinner } from '../components/Notices'
import { SolutionPanel } from '../components/SolutionPanel'
import { JsonPanel } from '../components/JsonPanel'
import { finishedPipes, isDeadEnd, legalTargets } from '../game/rules'
import type { Solution } from '../api/types'

export function PlayPage() {
  const { gameId } = useParams<{ gameId: string }>()
  const navigate = useNavigate()

  const game = useGame(gameId)
  const history = useGameMoves(gameId)
  const play = usePlayMoves(gameId)
  const rollBack = useRollBack(gameId)
  const reset = useResetGame(gameId)
  const solve = useSolveGame(gameId)

  const [selected, setSelected] = useState<number | null>(null)
  const [solution, setSolution] = useState<Solution | null>(null)
  const [rollbackSteps, setRollbackSteps] = useState(1)

  const pipes = game.data?.pipes ?? []
  const board = useMemo(() => pipes.map((pipe) => pipe.colors), [pipes])
  const targets = useMemo(
    () => (selected === null ? [] : legalTargets(board, selected)),
    [board, selected],
  )
  /** One pipe per color once the board is sorted: the goal to count against. */
  const colorsOnBoard = useMemo(() => new Set(board.flat()).size, [board])

  // The board moved under our feet: a stale selection would point at nothing.
  useEffect(() => setSelected(null), [game.data?.updated_at])
  // A solution describes the board it was asked about, not the one after it.
  useEffect(() => setSolution(null), [game.data?.moves_played])

  if (game.isPending) return <Spinner label="loading the game…" />
  if (game.isError || !game.data) {
    return (
      <div className="page">
        <ErrorNotice error={game.error} />
        <EmptyState title="This game could not be opened">
          <Link to="/games">back to the games</Link>
        </EmptyState>
      </div>
    )
  }

  const session = game.data
  const stuck = !session.solved && isDeadEnd(board)

  const click = (index: number) => {
    if (session.solved) return
    if (selected === null) {
      if (legalTargets(board, index).length) setSelected(index)
      return
    }
    if (selected === index) {
      setSelected(null)
      return
    }
    if (!targets.includes(index)) {
      // Treat it as picking a new source rather than as an illegal move.
      setSelected(legalTargets(board, index).length ? index : null)
      return
    }
    play.mutate([{ from: pipes[selected].id, to: pipes[index].id }], {
      onSettled: () => setSelected(null),
    })
  }

  const busy = play.isPending || rollBack.isPending || reset.isPending

  return (
    <div className="page play">
      <section className="panel">
        <header className="panel__header">
          <div>
            <h2>{session.name || 'Game'}</h2>
            <p className="panel__hint">
              <code>{session.id}</code>
              {session.scenario_id ? ' · from a saved scenario' : ' · from a submitted board'}
            </p>
          </div>
          <div className="panel__actions">
            <button type="button" onClick={() => navigate('/design')}>
              design another
            </button>
          </div>
        </header>

        <div className="stats">
          <div className={`stats__item ${session.solved ? 'stats__item--good' : ''}`}>
            <span className="stats__value">{session.status}</span>
            <span className="stats__label">status</span>
          </div>
          <div className="stats__item">
            <span className="stats__value">{session.moves_played}</span>
            <span className="stats__label">moves played</span>
          </div>
          <div className="stats__item">
            <span className="stats__value">
              {finishedPipes(board)} / {colorsOnBoard}
            </span>
            <span className="stats__label">colors sorted</span>
          </div>
          <div className="stats__item">
            <span className="stats__value">{session.completed_pipes}</span>
            <span className="stats__label">
              <code>completed_pipes</code>
            </span>
          </div>
        </div>

        {session.solved ? (
          <div className="notice notice--ok">
            Solved in {session.moves_played} moves. Roll back or reset to try a shorter line.
          </div>
        ) : null}
        {stuck ? (
          <div className="notice notice--warn">
            No legal move is left on this board — roll back a move or reset the game.
          </div>
        ) : null}

        <ErrorNotice error={play.error} onDismiss={() => play.reset()} />
        <ErrorNotice error={rollBack.error} onDismiss={() => rollBack.reset()} />

        <p className="play__hint">
          {selected === null
            ? 'Click the pipe to pour from.'
            : `Pouring from ${pipes[selected].label} — click a highlighted pipe, or click it again to cancel.`}
        </p>

        <BoardView
          pipes={pipes}
          selected={selected}
          targets={targets}
          onPipeClick={click}
          disabled={busy || session.solved}
        />

        <div className="play__controls">
          <button
            type="button"
            disabled={busy || session.moves_played === 0}
            onClick={() => rollBack.mutate(1)}
          >
            ↶ undo the last move
          </button>
          <label className="play__steps">
            <input
              type="number"
              min={1}
              max={Math.max(1, session.moves_played)}
              value={rollbackSteps}
              onChange={(event) => setRollbackSteps(Math.max(1, Number(event.target.value)))}
            />
            <button
              type="button"
              disabled={busy || session.moves_played === 0}
              onClick={() => rollBack.mutate(rollbackSteps)}
            >
              roll back
            </button>
          </label>
          <button
            type="button"
            disabled={busy || session.moves_played === 0}
            onClick={() => reset.mutate()}
          >
            ⟲ reset
          </button>
          <button
            type="button"
            className="button--primary"
            disabled={solve.isPending || session.solved}
            onClick={() => solve.mutate(undefined, { onSuccess: setSolution })}
          >
            {solve.isPending ? <Spinner label="solving…" /> : '⚡ solve from here'}
          </button>
        </div>

        <ErrorNotice error={solve.error} onDismiss={() => solve.reset()} />
      </section>

      {solution ? (
        <SolutionPanel
          solution={solution}
          pipes={pipes}
          applying={play.isPending}
          applyLabel="play every move on this game"
          onApply={(moves) => play.mutate(moves)}
        />
      ) : null}

      <section className="panel">
        <header className="panel__header">
          <div>
            <h3>History</h3>
            <p className="panel__hint">
              A rolled back move leaves this list for good — the next move takes its number.
            </p>
          </div>
        </header>

        {history.isPending ? <Spinner label="loading the history…" /> : null}
        {history.data && history.data.length === 0 ? (
          <p className="panel__hint">No move played yet.</p>
        ) : null}

        <ol className="history">
          {history.data?.map((move) => (
            <li key={move.id} className="history__item">
              <span className="history__index">{move.sequence}</span>
              <span className="history__notation">{move.notation}</span>
              {move.board_after ? (
                <BoardView
                  pipes={move.board_after.map((pipe) => ({
                    id: pipe.id,
                    label: pipe.label,
                    colors: pipe.colors,
                  }))}
                  compact
                />
              ) : null}
            </li>
          ))}
        </ol>
      </section>

      <JsonPanel
        title="The game, as the API sees it"
        hint={`GET /api/v1/games/${session.id}`}
        value={session}
        fileName="game.json"
      />
    </div>
  )
}
