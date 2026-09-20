/**
 * Play a game session.
 *
 * Every move goes to the API and the board that comes back is the one drawn —
 * the client never decides the outcome of a pour. The local rules are used for
 * one thing only: lighting up the pipes a selected pipe may pour into, so the
 * board answers the pointer without a round trip.
 */

import { useEffect, useMemo, useState } from 'react'
import {
  Ban,
  CircleCheck,
  PartyPopper,
  Pencil,
  RotateCcw,
  Undo2,
  Zap,
} from 'lucide-react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'

import {
  useGame,
  useGameMoves,
  usePlayMoves,
  useResetGame,
  useRollBack,
  useSolveGame,
} from '@/api/queries'
import { Board } from '@/components/board'
import { JsonCard } from '@/components/json-card'
import { SolutionCard } from '@/components/solution-card'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { Input } from '@/components/ui/input'
import { Item, ItemContent, ItemGroup, ItemMedia, ItemTitle } from '@/components/ui/item'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import { Spinner } from '@/components/ui/spinner'
import type { Solution } from '@/api/types'
import { finishedPipes, isDeadEnd, legalTargets } from '@/game/rules'
import { errorCode, explain } from '@/lib/errors'

function Stat({ value, label }: { value: React.ReactNode; label: React.ReactNode }) {
  return (
    <Card className="flex-1 gap-0 py-4">
      <CardContent className="px-4">
        <p className="text-xl font-semibold">{value}</p>
        <p className="text-muted-foreground text-xs">{label}</p>
      </CardContent>
    </Card>
  )
}

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

  const pipes = useMemo(() => game.data?.pipes ?? [], [game.data])
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

  const fail = (error: unknown) => toast.error(errorCode(error), { description: explain(error) })

  if (game.isPending) {
    return (
      <div className="flex flex-col gap-4">
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-48 w-full" />
      </div>
    )
  }

  if (game.isError || !game.data) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Ban />
          </EmptyMedia>
          <EmptyTitle>This game could not be opened</EmptyTitle>
          <EmptyDescription>{explain(game.error)}</EmptyDescription>
        </EmptyHeader>
        <Button variant="outline" asChild>
          <Link to="/games">Back to the games</Link>
        </Button>
      </Empty>
    )
  }

  const session = game.data
  const stuck = !session.solved && isDeadEnd(board)
  const busy = play.isPending || rollBack.isPending || reset.isPending

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
      onError: fail,
    })
  }

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>{session.name || 'Game'}</CardTitle>
          <CardDescription className="font-mono text-xs">
            {session.id} · {session.scenario_id ? 'from a saved scenario' : 'from a submitted board'}
          </CardDescription>
          <CardAction>
            <Button variant="outline" size="sm" onClick={() => navigate('/design')}>
              <Pencil />
              Design another
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent className="flex flex-col gap-5">
          <div className="flex flex-wrap gap-3">
            <Stat
              value={<Badge variant={session.solved ? 'default' : 'secondary'}>{session.status}</Badge>}
              label="status"
            />
            <Stat value={session.moves_played} label="moves played" />
            <Stat value={`${finishedPipes(board)} / ${colorsOnBoard}`} label="colors sorted" />
            <Stat value={session.completed_pipes} label={<code>completed_pipes</code>} />
          </div>

          {session.solved ? (
            <Alert>
              <PartyPopper />
              <AlertTitle>Solved in {session.moves_played} moves</AlertTitle>
              <AlertDescription>Roll back or reset to try a shorter line.</AlertDescription>
            </Alert>
          ) : null}

          {stuck ? (
            <Alert variant="destructive">
              <Ban />
              <AlertTitle>No legal move is left on this board</AlertTitle>
              <AlertDescription>Roll back a move, or reset the game.</AlertDescription>
            </Alert>
          ) : null}

          <p className="text-muted-foreground text-sm">
            {selected === null
              ? 'Click the pipe to pour from.'
              : `Pouring from ${pipes[selected].label} — click a highlighted pipe, or click it again to cancel.`}
          </p>

          <Board
            pipes={pipes}
            selected={selected}
            targets={targets}
            onPipeClick={click}
            disabled={busy || session.solved}
          />
        </CardContent>

        <CardFooter className="flex-wrap gap-2">
          <Button
            variant="outline"
            disabled={busy || session.moves_played === 0}
            onClick={() => rollBack.mutate(1, { onError: fail })}
          >
            <Undo2 />
            Undo the last move
          </Button>

          <ButtonGroup>
            <Input
              type="number"
              min={1}
              max={Math.max(1, session.moves_played)}
              value={rollbackSteps}
              className="w-20"
              aria-label="moves to roll back"
              onChange={(event) => setRollbackSteps(Math.max(1, Number(event.target.value)))}
            />
            <Button
              variant="outline"
              disabled={busy || session.moves_played === 0}
              onClick={() => rollBack.mutate(rollbackSteps, { onError: fail })}
            >
              Roll back
            </Button>
          </ButtonGroup>

          <Button
            variant="outline"
            disabled={busy || session.moves_played === 0}
            onClick={() => reset.mutate(undefined, { onError: fail })}
          >
            <RotateCcw />
            Reset
          </Button>

          <Button
            className="ml-auto"
            disabled={solve.isPending || session.solved}
            onClick={() => solve.mutate(undefined, { onSuccess: setSolution, onError: fail })}
          >
            {solve.isPending ? <Spinner /> : <Zap />}
            Solve from here
          </Button>
        </CardFooter>
      </Card>

      {solution ? (
        <SolutionCard
          solution={solution}
          pipes={pipes}
          applying={play.isPending}
          applyLabel="Play every move on this game"
          onApply={(moves) => play.mutate(moves, { onError: fail })}
        />
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle>History</CardTitle>
          <CardDescription>
            A rolled back move leaves this list for good — the next move takes its number.
          </CardDescription>
        </CardHeader>

        <CardContent>
          {history.isPending ? <Skeleton className="h-24 w-full" /> : null}

          {history.data && history.data.length === 0 ? (
            <Empty className="border border-dashed">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <CircleCheck />
                </EmptyMedia>
                <EmptyTitle>No move played yet</EmptyTitle>
                <EmptyDescription>Pour from one pipe into another to get going.</EmptyDescription>
              </EmptyHeader>
            </Empty>
          ) : null}

          {history.data && history.data.length > 0 ? (
            <ScrollArea className="h-96 rounded-md border">
              <ItemGroup className="p-2">
                {history.data.map((move) => (
                  <Item key={move.id} variant="muted" className="mb-2">
                    <ItemMedia>
                      <Badge variant="outline" className="font-mono">
                        {move.sequence}
                      </Badge>
                    </ItemMedia>
                    <ItemContent>
                      <ItemTitle className="font-mono">{move.notation}</ItemTitle>
                      {move.board_after ? (
                        <Board
                          pipes={move.board_after.map((pipe) => ({
                            id: pipe.id,
                            label: pipe.label,
                            colors: pipe.colors,
                          }))}
                          size="sm"
                        />
                      ) : null}
                    </ItemContent>
                  </Item>
                ))}
              </ItemGroup>
            </ScrollArea>
          ) : null}
        </CardContent>
      </Card>

      <JsonCard
        title="The game, as the API sees it"
        description={`GET /api/v1/games/${session.id}`}
        value={session}
        fileName="game.json"
      />
    </div>
  )
}
