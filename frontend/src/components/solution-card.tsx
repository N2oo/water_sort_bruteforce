/**
 * A solution, step by step.
 *
 * The API answers with pipe uuids *and* labels. A solve of an unsaved board
 * mints fresh uuids on every call, so a move is resolved by label first and by
 * uuid second — that way the same card replays the solution of a puzzle, of a
 * saved scenario and of a running game.
 */

import { useEffect, useMemo, useState } from 'react'
import {
  ChevronLeft,
  ChevronRight,
  Pause,
  Play,
  SkipBack,
  SkipForward,
  SearchX,
} from 'lucide-react'

import { Board, type BoardPipe } from '@/components/board'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Slider } from '@/components/ui/slider'
import { Spinner } from '@/components/ui/spinner'
import type { Solution } from '@/api/types'
import { applyMove } from '@/game/rules'

interface SolutionCardProps {
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

export function SolutionCard({
  solution,
  pipes,
  onApply,
  applyLabel = 'Play this solution',
  applying = false,
}: SolutionCardProps) {
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
      <Card>
        <CardContent>
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <SearchX />
              </EmptyMedia>
              <EmptyTitle>No solution</EmptyTitle>
              <EmptyDescription>
                The solver explored the whole search space and found no way to finish this board.
                Check that every color comes in multiples of four, and that there is a spare pipe to
                work with.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        </CardContent>
      </Card>
    )
  }

  const board = boards[Math.min(step, boards.length - 1)]
  const nextMove = moves[step]
  const shownPipes: BoardPipe[] = pipes.map((pipe, index) => ({
    ...pipe,
    colors: board[index] ?? [],
  }))

  return (
    <Card>
      <CardHeader>
        <CardTitle>Solution — {solution.moves_count} moves</CardTitle>
        <CardDescription>
          Step {step} of {solution.moves_count}
          {nextMove ? ` · next: ${nextMove.notation}` : ' · solved'}
        </CardDescription>
        {onApply ? (
          <CardAction>
            <Button
              disabled={applying}
              onClick={() => onApply(solution.moves.map((move) => ({ from: move.from, to: move.to })))}
            >
              {applying ? <Spinner /> : <Play />}
              {applyLabel}
            </Button>
          </CardAction>
        ) : null}
      </CardHeader>

      <CardContent className="flex flex-col gap-4">
        <Board
          pipes={shownPipes}
          highlight={nextMove ? { from: nextMove.fromIndex, to: nextMove.toIndex } : null}
          size="sm"
        />

        <div className="flex flex-wrap items-center gap-3">
          <ButtonGroup>
            <Button variant="outline" size="icon" onClick={() => setStep(0)} disabled={step === 0} aria-label="first step">
              <SkipBack />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setStep((current) => Math.max(0, current - 1))}
              disabled={step === 0}
              aria-label="previous move"
            >
              <ChevronLeft />
            </Button>
            <Button variant="outline" onClick={() => setPlaying((current) => !current)}>
              {playing ? <Pause /> : <Play />}
              {playing ? 'Pause' : 'Play'}
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setStep((current) => Math.min(moves.length, current + 1))}
              disabled={step >= moves.length}
              aria-label="next move"
            >
              <ChevronRight />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setStep(moves.length)}
              disabled={step >= moves.length}
              aria-label="last step"
            >
              <SkipForward />
            </Button>
          </ButtonGroup>

          <Slider
            className="min-w-40 flex-1"
            min={0}
            max={moves.length}
            step={1}
            value={[step]}
            onValueChange={([value]) => setStep(value)}
            aria-label="solution step"
          />
        </div>

        <ScrollArea className="h-40 rounded-md border">
          <div className="flex flex-wrap gap-1.5 p-2">
            {moves.map((move, index) => (
              <Badge
                key={index}
                asChild
                variant={index === step ? 'default' : index < step ? 'secondary' : 'outline'}
              >
                <button type="button" onClick={() => setStep(index)} className="font-mono">
                  <span className="opacity-60">{index + 1}</span>
                  {move.notation}
                </button>
              </Badge>
            ))}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  )
}
