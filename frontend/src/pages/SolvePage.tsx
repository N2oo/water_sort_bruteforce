/**
 * The solver workspace.
 *
 * Three ways in — the board being designed, a saved scenario, or JSON pasted
 * straight in — and one way out: the solution, replayable step by step, with
 * the option to open it as a real game.
 */

import { useEffect, useMemo, useState } from 'react'
import { AlertTriangle, PencilLine, Play, XCircle, Zap } from 'lucide-react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { toast } from 'sonner'

import { useCreateGame, useScenarios, useSolvePuzzle, useSolveScenario } from '@/api/queries'
import { Board } from '@/components/board'
import { JsonCard } from '@/components/json-card'
import { SolutionCard } from '@/components/solution-card'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { Spinner } from '@/components/ui/spinner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import type { Solution } from '@/api/types'
import { errorCode, explain } from '@/lib/errors'
import {
  ImportError,
  draftFromJson,
  draftFromPipes,
  hasErrors,
  toGamePayload,
  toSolvePayload,
  validate,
} from '@/game/scenario'
import { useDraft } from '@/state/draft'

type Source = 'draft' | 'scenario' | 'json'

export function SolvePage() {
  const navigate = useNavigate()
  const [params, setParams] = useSearchParams()
  const { draft, setDraft } = useDraft()

  const scenarioParam = params.get('scenario') ?? ''
  const [source, setSource] = useState<Source>(scenarioParam ? 'scenario' : 'draft')
  const [scenarioId, setScenarioId] = useState(scenarioParam)
  const [jsonText, setJsonText] = useState('')
  const [solution, setSolution] = useState<Solution | null>(null)

  const scenarios = useScenarios({ limit: 100 })
  const solvePuzzle = useSolvePuzzle()
  const solveScenario = useSolveScenario()
  const createGame = useCreateGame()

  /** The board the solution is about, whatever the source is. */
  const board = useMemo(() => {
    if (source === 'scenario') {
      const scenario = scenarios.data?.find((entry) => entry.id === scenarioId)
      return scenario
        ? draftFromPipes(scenario.pipes, scenario.name, scenario.description ?? '')
        : null
    }
    if (source === 'json') {
      if (!jsonText.trim()) return null
      try {
        return draftFromJson(jsonText)
      } catch {
        return null
      }
    }
    return draft
  }, [source, scenarioId, scenarios.data, jsonText, draft])

  const problems = useMemo(() => (board ? validate(board) : []), [board])
  const blocked = !board || hasErrors(problems)
  const pending = solvePuzzle.isPending || solveScenario.isPending

  // A fresh board invalidates the solution shown next to it.
  useEffect(() => setSolution(null), [source, scenarioId, jsonText, draft])

  const fail = (error: unknown) => toast.error(errorCode(error), { description: explain(error) })

  const run = () => {
    if (!board) return
    if (source === 'scenario' && scenarioId) {
      solveScenario.mutate(scenarioId, { onSuccess: setSolution, onError: fail })
      return
    }
    solvePuzzle.mutate(toSolvePayload(board).puzzle, { onSuccess: setSolution, onError: fail })
  }

  const play = () => {
    if (!board) return
    createGame.mutate(
      source === 'scenario' && scenarioId
        ? { scenario_id: scenarioId, name: board.name || undefined }
        : toGamePayload(board),
      { onSuccess: (game) => navigate(`/games/${game.id}`), onError: fail },
    )
  }

  const openInDesigner = () => {
    try {
      setDraft(draftFromJson(jsonText))
      navigate('/design')
    } catch (error) {
      toast.error('That JSON could not be read', {
        description: error instanceof ImportError ? error.message : String(error),
      })
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>2 · Generate a solution</CardTitle>
          <CardDescription>
            The solver is the original exhaustive search. It answers in milliseconds on the shipped
            levels; a hostile board can hit <code className="font-mono">SOLVER_TIMEOUT_SECONDS</code>{' '}
            instead.
          </CardDescription>
          <CardAction className="flex flex-wrap gap-2">
            <Button disabled={blocked || pending} onClick={run}>
              {pending ? <Spinner /> : <Zap />}
              Solve this board
            </Button>
            <Button
              variant="secondary"
              disabled={blocked || createGame.isPending}
              onClick={play}
            >
              {createGame.isPending ? <Spinner /> : <Play />}
              Play it instead
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent className="flex flex-col gap-5">
          <Tabs value={source} onValueChange={(value) => setSource(value as Source)}>
            <TabsList className="w-full sm:w-fit">
              <TabsTrigger value="draft">Designed board</TabsTrigger>
              <TabsTrigger value="scenario">Saved scenario</TabsTrigger>
              <TabsTrigger value="json">Pasted JSON</TabsTrigger>
            </TabsList>

            <TabsContent value="scenario" className="pt-4">
              {scenarios.isPending ? (
                <Skeleton className="h-9 w-72" />
              ) : (
                <Field>
                  <FieldLabel htmlFor="scenario-picker">Saved scenario</FieldLabel>
                  <Select
                    value={scenarioId}
                    onValueChange={(value) => {
                      setScenarioId(value)
                      setParams(value ? { scenario: value } : {})
                    }}
                  >
                    <SelectTrigger id="scenario-picker" className="w-full sm:w-96">
                      <SelectValue placeholder="Pick a saved scenario…" />
                    </SelectTrigger>
                    <SelectContent>
                      {scenarios.data?.map((scenario) => (
                        <SelectItem key={scenario.id} value={scenario.id}>
                          {scenario.name} · {scenario.pipes.length} pipes
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FieldDescription>
                    Solved through <code className="font-mono">POST /scenarios/{'{id}'}/solve</code>,
                    from its initial board.
                  </FieldDescription>
                </Field>
              )}
            </TabsContent>

            <TabsContent value="json" className="pt-4">
              <Field>
                <FieldLabel htmlFor="solve-json">Board</FieldLabel>
                <Textarea
                  id="solve-json"
                  rows={7}
                  className="font-mono text-xs"
                  value={jsonText}
                  placeholder='{"pipes": {"P1": ["Blue","Blue","Red","Red"], "P2": ["Red","Red","Blue","Blue"], "P3": []}}'
                  onChange={(event) => setJsonText(event.target.value)}
                />
                <Button variant="outline" disabled={!jsonText.trim()} onClick={openInDesigner}>
                  <PencilLine />
                  Open it in the designer
                </Button>
              </Field>
            </TabsContent>
          </Tabs>

          {board ? (
            <div className="flex flex-col gap-3">
              <h3 className="text-sm font-medium">{board.name || 'Untitled board'}</h3>
              <Board pipes={board.pipes} size="sm" />
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">Pick a board above to solve it.</p>
          )}

          {problems.map((problem, index) => (
            <Alert key={index} variant={problem.severity === 'error' ? 'destructive' : 'default'}>
              {problem.severity === 'error' ? <XCircle /> : <AlertTriangle />}
              <AlertTitle>
                {problem.severity === 'error' ? 'The API would refuse this board' : 'This board has no solution'}
              </AlertTitle>
              <AlertDescription>{problem.message}</AlertDescription>
            </Alert>
          ))}
        </CardContent>
      </Card>

      {solution && board ? <SolutionCard solution={solution} pipes={board.pipes} /> : null}

      <div className="grid gap-6 lg:grid-cols-2">
        {solution?.solved ? (
          <JsonCard
            title="The solution, as the API returned it"
            description="the moves can be posted back to POST /api/v1/games/{id}/moves"
            value={solution}
            fileName="solution.json"
          />
        ) : null}
        {board ? (
          <JsonCard
            title="Solve payload"
            description="POST /api/v1/puzzles/solve"
            value={toSolvePayload(board)}
            fileName="solve-request.json"
          />
        ) : null}
      </div>
    </div>
  )
}
