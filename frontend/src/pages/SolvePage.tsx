/**
 * The solver workspace.
 *
 * Three ways in — the board being designed, a saved scenario, or JSON pasted
 * straight in — and one way out: the solution, replayable step by step, with
 * the option to open it as a real game.
 */

import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'

import { useCreateGame, useScenarios, useSolvePuzzle, useSolveScenario } from '../api/queries'
import { BoardView } from '../components/BoardView'
import { ErrorNotice, ProblemList, Spinner } from '../components/Notices'
import { JsonPanel } from '../components/JsonPanel'
import { SolutionPanel } from '../components/SolutionPanel'
import {
  ImportError,
  draftFromJson,
  draftFromPipes,
  hasErrors,
  toGamePayload,
  toSolvePayload,
  validate,
} from '../game/scenario'
import { useDraft } from '../state/draft'
import type { Solution } from '../api/types'

type Source = 'draft' | 'scenario' | 'json'

export function SolvePage() {
  const navigate = useNavigate()
  const [params, setParams] = useSearchParams()
  const { draft, setDraft } = useDraft()

  const scenarioParam = params.get('scenario') ?? ''
  const [source, setSource] = useState<Source>(scenarioParam ? 'scenario' : 'draft')
  const [scenarioId, setScenarioId] = useState(scenarioParam)
  const [jsonText, setJsonText] = useState('')
  const [jsonError, setJsonError] = useState<string | null>(null)
  const [solution, setSolution] = useState<Solution | null>(null)

  const scenarios = useScenarios({ limit: 100 })
  const solvePuzzle = useSolvePuzzle()
  const solveScenario = useSolveScenario()
  const createGame = useCreateGame()

  /** The board the solution is about, whatever the source is. */
  const board = useMemo(() => {
    if (source === 'scenario') {
      const scenario = scenarios.data?.find((entry) => entry.id === scenarioId)
      return scenario ? draftFromPipes(scenario.pipes, scenario.name, scenario.description ?? '') : null
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
  const error = solvePuzzle.error ?? solveScenario.error

  // A fresh board invalidates the solution shown next to it.
  useEffect(() => setSolution(null), [source, scenarioId, jsonText, draft])

  const run = () => {
    if (!board) return
    if (source === 'scenario' && scenarioId) {
      solveScenario.mutate(scenarioId, { onSuccess: setSolution })
      return
    }
    solvePuzzle.mutate(toSolvePayload(board).puzzle, { onSuccess: setSolution })
  }

  const play = () => {
    if (!board) return
    createGame.mutate(
      source === 'scenario' && scenarioId
        ? { scenario_id: scenarioId, name: board.name || undefined }
        : toGamePayload(board),
      { onSuccess: (game) => navigate(`/games/${game.id}`) },
    )
  }

  const loadJson = () => {
    try {
      setDraft(draftFromJson(jsonText))
      setSource('draft')
      setJsonError(null)
      navigate('/design')
    } catch (cause) {
      setJsonError(cause instanceof ImportError ? cause.message : String(cause))
    }
  }

  return (
    <div className="page solve">
      <section className="panel">
        <header className="panel__header">
          <div>
            <h2>2 · Generate a solution</h2>
            <p className="panel__hint">
              The solver is the original exhaustive search. It answers in milliseconds on the shipped
              levels; a hostile board can hit <code>SOLVER_TIMEOUT_SECONDS</code> instead.
            </p>
          </div>
          <div className="panel__actions">
            <button type="button" className="button--primary" disabled={blocked || pending} onClick={run}>
              {pending ? <Spinner label="solving…" /> : '⚡ Solve this board'}
            </button>
            <button type="button" disabled={blocked || createGame.isPending} onClick={play}>
              {createGame.isPending ? 'starting…' : '▶ Play it instead'}
            </button>
          </div>
        </header>

        <div className="tabs" role="tablist">
          {(
            [
              ['draft', 'the board I designed'],
              ['scenario', 'a saved scenario'],
              ['json', 'JSON I paste'],
            ] as Array<[Source, string]>
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              role="tab"
              aria-selected={source === value}
              className={`tabs__tab ${source === value ? 'tabs__tab--active' : ''}`}
              onClick={() => setSource(value)}
            >
              {label}
            </button>
          ))}
        </div>

        {source === 'scenario' ? (
          <div className="solve__scenario">
            <select
              value={scenarioId}
              onChange={(event) => {
                setScenarioId(event.target.value)
                setParams(event.target.value ? { scenario: event.target.value } : {})
              }}
            >
              <option value="">— pick a saved scenario —</option>
              {scenarios.data?.map((scenario) => (
                <option key={scenario.id} value={scenario.id}>
                  {scenario.name} · {scenario.pipes.length} pipes
                </option>
              ))}
            </select>
            {scenarios.isPending ? <Spinner label="loading scenarios…" /> : null}
            <ErrorNotice error={scenarios.error} />
          </div>
        ) : null}

        {source === 'json' ? (
          <div className="import">
            <textarea
              rows={8}
              value={jsonText}
              placeholder='{"pipes": {"P1": ["Blue", "Blue", "Red", "Red"], "P2": ["Red", "Red", "Blue", "Blue"], "P3": []}}'
              onChange={(event) => {
                setJsonText(event.target.value)
                setJsonError(null)
              }}
            />
            <div className="import__actions">
              <button type="button" disabled={!jsonText.trim()} onClick={loadJson}>
                open it in the designer
              </button>
            </div>
            {jsonError ? <ErrorNotice error={new Error(jsonError)} onDismiss={() => setJsonError(null)} /> : null}
          </div>
        ) : null}

        {board ? (
          <>
            <h3 className="solve__board-title">{board.name || 'Untitled board'}</h3>
            <BoardView pipes={board.pipes} compact />
            <ProblemList problems={problems} />
          </>
        ) : (
          <p className="panel__hint">Pick a board above to solve it.</p>
        )}

        <ErrorNotice
          error={error}
          onDismiss={() => {
            solvePuzzle.reset()
            solveScenario.reset()
          }}
        />
        <ErrorNotice error={createGame.error} onDismiss={() => createGame.reset()} />
      </section>

      {solution && board ? (
        <SolutionPanel solution={solution} pipes={board.pipes} />
      ) : null}

      {solution?.solved ? (
        <JsonPanel
          title="The solution, as the API returned it"
          hint="the moves can be posted straight back to POST /api/v1/games/{id}/moves"
          value={solution}
          fileName="solution.json"
        />
      ) : null}

      {board ? (
        <JsonPanel
          title="Solve payload"
          hint="POST /api/v1/puzzles/solve"
          value={toSolvePayload(board)}
          fileName="solve-request.json"
        />
      ) : null}
    </div>
  )
}
