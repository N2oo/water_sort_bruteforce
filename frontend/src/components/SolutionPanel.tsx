/**
 * A solution, step by step.
 *
 * The API answers with pipe uuids *and* labels. A solve of an unsaved board
 * mints fresh uuids on every call, so the stepper resolves a move by label
 * first and falls back on the uuid — that way the same component replays the
 * solution of a puzzle, of a saved scenario and of a running game.
 */

import { useEffect, useMemo, useState } from 'react'

import type { Solution } from '../api/types'
import { applyMove } from '../game/rules'
import { BoardView, type BoardPipe } from './BoardView'

interface SolutionPanelProps {
  solution: Solution
  /** The board the solution starts from. */
  pipes: BoardPipe[]
  /** Offered when the solution can be posted straight back to a game. */
  onApply?: (moves: Array<{ from: string; to: string }>) => void
  applyLabel?: string
  applying?: boolean
}

/** Where each move lands on the board, by index. */
function resolveMoves(solution: Solution, pipes: BoardPipe[]) {
  const byLabel = new Map(pipes.map((pipe, index) => [pipe.label, index]))
  const byId = new Map(pipes.map((pipe, index) => [pipe.id ?? '', index]))

  return solution.moves.map((move) => ({
    ...move,
    fromIndex: byLabel.get(move.from_label) ?? byId.get(move.from) ?? -1,
    toIndex: byLabel.get(move.to_label) ?? byId.get(move.to) ?? -1,
  }))
}

export function SolutionPanel({
  solution,
  pipes,
  onApply,
  applyLabel = 'play this solution',
  applying = false,
}: SolutionPanelProps) {
  const [step, setStep] = useState(0)
  const [playing, setPlaying] = useState(false)

  const moves = useMemo(() => resolveMoves(solution, pipes), [solution, pipes])

  /** Board after every prefix of the solution: `boards[n]` is `n` moves in. */
  const boards = useMemo(() => {
    const states: string[][][] = [pipes.map((pipe) => [...pipe.colors])]
    for (const move of moves) {
      const current = states[states.length - 1]
      const next =
        move.fromIndex >= 0 && move.toIndex >= 0
          ? applyMove(current, move.fromIndex, move.toIndex)
          : null
      states.push(next ?? current.map((pipe) => [...pipe]))
    }
    return states
  }, [moves, pipes])

  useEffect(() => setStep(0), [solution])

  useEffect(() => {
    if (!playing) return
    if (step >= moves.length) {
      setPlaying(false)
      return
    }
    const timer = window.setTimeout(() => setStep((current) => current + 1), 450)
    return () => window.clearTimeout(timer)
  }, [playing, step, moves.length])

  if (!solution.solved) {
    return (
      <section className="panel">
        <header className="panel__header">
          <h3>No solution</h3>
        </header>
        <p className="panel__hint">
          The solver explored the whole search space and found no way to finish this board. Check
          that every color comes in multiples of four, and that there is a spare pipe to work with.
        </p>
      </section>
    )
  }

  const board = boards[Math.min(step, boards.length - 1)]
  const nextMove = moves[step]
  const shownPipes: BoardPipe[] = pipes.map((pipe, index) => ({
    ...pipe,
    colors: board[index] ?? [],
  }))

  return (
    <section className="panel solution">
      <header className="panel__header">
        <div>
          <h3>Solution — {solution.moves_count} moves</h3>
          <p className="panel__hint">
            step {step} / {solution.moves_count}
            {nextMove ? ` — next: ${nextMove.notation}` : ' — solved'}
          </p>
        </div>
        {onApply ? (
          <div className="panel__actions">
            <button
              type="button"
              className="button--primary"
              disabled={applying}
              onClick={() => onApply(solution.moves.map((move) => ({ from: move.from, to: move.to })))}
            >
              {applying ? 'playing…' : applyLabel}
            </button>
          </div>
        ) : null}
      </header>

      <BoardView
        pipes={shownPipes}
        highlight={nextMove ? { from: nextMove.fromIndex, to: nextMove.toIndex } : null}
        compact
      />

      <div className="solution__controls">
        <button type="button" onClick={() => setStep(0)} disabled={step === 0}>
          ⏮
        </button>
        <button
          type="button"
          onClick={() => setStep((current) => Math.max(0, current - 1))}
          disabled={step === 0}
        >
          ◀
        </button>
        <button type="button" onClick={() => setPlaying((current) => !current)}>
          {playing ? '⏸ pause' : '▶ play'}
        </button>
        <button
          type="button"
          onClick={() => setStep((current) => Math.min(moves.length, current + 1))}
          disabled={step >= moves.length}
        >
          ▶
        </button>
        <button
          type="button"
          onClick={() => setStep(moves.length)}
          disabled={step >= moves.length}
        >
          ⏭
        </button>
        <input
          type="range"
          min={0}
          max={moves.length}
          value={step}
          onChange={(event) => setStep(Number(event.target.value))}
          aria-label="solution step"
        />
      </div>

      <ol className="solution__moves">
        {moves.map((move, index) => (
          <li key={index}>
            <button
              type="button"
              className={`solution__move ${index === step ? 'solution__move--current' : ''} ${
                index < step ? 'solution__move--done' : ''
              }`}
              onClick={() => setStep(index)}
            >
              <span className="solution__move-index">{index + 1}</span>
              {move.notation}
            </button>
          </li>
        ))}
      </ol>
    </section>
  )
}
