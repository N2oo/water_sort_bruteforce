/**
 * Design a scenario, then choose: play it, or have it solved.
 *
 * Nothing here needs the API. The board is edited locally, validated against
 * the same rules the backend enforces, and shown as the exact JSON payload the
 * API accepts — so a scenario can be taken away and curl'd by hand. The two
 * buttons at the top are the fork the whole app is built around.
 */

import { useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'

import { useCreateGame, useCreateScenario } from '../api/queries'
import { API_BASE_URL } from '../api/client'
import { BoardView } from '../components/BoardView'
import { ErrorNotice, ProblemList } from '../components/Notices'
import { JsonPanel } from '../components/JsonPanel'
import { COLOR_NAMES, colorStyle, type ColorName } from '../game/colors'
import { DEFAULT_GENERATOR, generateBoard, type GeneratorOptions } from '../game/generator'
import { PIPE_CAPACITY } from '../game/rules'
import { SAMPLES } from '../game/samples'
import {
  ImportError,
  countByColor,
  draftFromJson,
  emptyDraft,
  hasErrors,
  relabel,
  toGamePayload,
  toLevelDocument,
  toScenarioPayload,
  toSolvePayload,
  validate,
  type Draft,
} from '../game/scenario'
import { useDraft } from '../state/draft'

type Brush = ColorName | 'erase'

export function DesignerPage() {
  const navigate = useNavigate()
  const { draft, setDraft, updateDraft, resetDraft } = useDraft()

  const [brush, setBrush] = useState<Brush>('Blue')
  const [saveName, setSaveName] = useState('')
  const [saveOnPlay, setSaveOnPlay] = useState(false)
  const [importText, setImportText] = useState('')
  const [importError, setImportError] = useState<string | null>(null)
  const [generator, setGenerator] = useState<GeneratorOptions>(DEFAULT_GENERATOR)
  const fileInput = useRef<HTMLInputElement>(null)

  const createGame = useCreateGame()
  const createScenario = useCreateScenario()

  const problems = useMemo(() => validate(draft), [draft])
  const blocked = hasErrors(problems)
  const counts = useMemo(() => countByColor(draft), [draft])

  /* ------------------------------------------------------------- editing */

  const paint = (index: number) => {
    updateDraft((current) => {
      const pipes = current.pipes.map((pipe, position) => {
        if (position !== index) return pipe
        if (brush === 'erase') return { ...pipe, colors: pipe.colors.slice(0, -1) }
        if (pipe.colors.length >= PIPE_CAPACITY) return pipe
        return { ...pipe, colors: [...pipe.colors, brush] }
      })
      return { ...current, pipes }
    })
  }

  const clearPipe = (index: number) =>
    updateDraft((current) => ({
      ...current,
      pipes: current.pipes.map((pipe, position) =>
        position === index ? { ...pipe, colors: [] } : pipe,
      ),
    }))

  const removePipe = (index: number) =>
    updateDraft((current) => ({
      ...current,
      pipes: relabel(current.pipes.filter((_, position) => position !== index)),
    }))

  const addPipe = () =>
    updateDraft((current) => ({
      ...current,
      pipes: [...current.pipes, { label: `P${current.pipes.length + 1}`, colors: [] }],
    }))

  const load = (next: Draft) => {
    setDraft(next)
    setImportError(null)
  }

  const importJson = (text: string) => {
    try {
      load(draftFromJson(text))
      setImportText('')
    } catch (error) {
      setImportError(error instanceof ImportError ? error.message : String(error))
    }
  }

  const openFile = async (file: File | undefined) => {
    if (!file) return
    importJson(await file.text())
  }

  /* -------------------------------------------------------------- the fork */

  const play = () => {
    // `save_as_scenario` lets the API store the board in the same call that
    // starts the game, so a board played once can be replayed later.
    const keep = saveOnPlay ? draft.name.trim() || 'Untitled scenario' : undefined
    createGame.mutate(toGamePayload(draft, keep), {
      onSuccess: (game) => navigate(`/games/${game.id}`),
    })
  }

  const solve = () => navigate('/solve')

  const save = () => {
    const payload = toScenarioPayload({ ...draft, name: saveName.trim() || draft.name })
    createScenario.mutate(payload, {
      onSuccess: (scenario) => {
        updateDraft((current) => ({ ...current, name: scenario.name }))
        setSaveName('')
      },
    })
  }

  const origin = API_BASE_URL || 'http://localhost:8080'

  return (
    <div className="page designer">
      <section className="panel">
        <header className="panel__header">
          <div>
            <h2>1 · Design your scenario</h2>
            <p className="panel__hint">
              Pick a color, click a pipe to pour one unit in. A pipe holds {PIPE_CAPACITY} units,
              bottom first — exactly what the API stores.
            </p>
          </div>
          <div className="panel__actions">
            <button type="button" className="button--primary" disabled={blocked || createGame.isPending} onClick={play}>
              {createGame.isPending ? 'starting…' : '▶ Play this scenario'}
            </button>
            <button type="button" className="button--primary" disabled={blocked} onClick={solve}>
              ⚡ Generate a solution
            </button>
          </div>
        </header>

        <label className="designer__keep">
          <input
            type="checkbox"
            checked={saveOnPlay}
            onChange={(event) => setSaveOnPlay(event.target.checked)}
          />
          save it as a scenario when I start the game
        </label>

        <ErrorNotice error={createGame.error} onDismiss={() => createGame.reset()} />

        <div className="designer__identity">
          <label>
            <span>Name</span>
            <input
              value={draft.name}
              placeholder="Level 145"
              onChange={(event) => updateDraft((current) => ({ ...current, name: event.target.value }))}
            />
          </label>
          <label>
            <span>Description</span>
            <input
              value={draft.description}
              placeholder="optional"
              onChange={(event) =>
                updateDraft((current) => ({ ...current, description: event.target.value }))
              }
            />
          </label>
        </div>

        <div className="palette">
          {COLOR_NAMES.map((color) => {
            const style = colorStyle(color)
            const used = counts.get(color) ?? 0
            return (
              <button
                key={color}
                type="button"
                className={`palette__swatch ${brush === color ? 'palette__swatch--active' : ''}`}
                style={{ background: style.fill, color: style.ink }}
                onClick={() => setBrush(color)}
                title={`${color} — ${used} unit(s) on the board`}
              >
                {color}
                <span className="palette__count">{used}</span>
              </button>
            )
          })}
          <button
            type="button"
            className={`palette__swatch palette__swatch--erase ${brush === 'erase' ? 'palette__swatch--active' : ''}`}
            onClick={() => setBrush('erase')}
            title="remove the top unit of the pipe you click"
          >
            erase
          </button>
        </div>

        <div className="designer__board">
          {draft.pipes.map((pipe, index) => (
            <div key={index} className="designer__pipe">
              <BoardView pipes={[pipe]} onPipeClick={() => paint(index)} compact />
              <div className="designer__pipe-actions">
                <button type="button" onClick={() => clearPipe(index)} title="empty this pipe">
                  empty
                </button>
                <button
                  type="button"
                  onClick={() => removePipe(index)}
                  title="remove this pipe from the board"
                  disabled={draft.pipes.length <= 2}
                >
                  remove
                </button>
              </div>
            </div>
          ))}
          <button type="button" className="designer__add" onClick={addPipe}>
            + add a pipe
          </button>
        </div>

        <ProblemList problems={problems} />

        <div className="designer__toolbar">
          <button type="button" onClick={() => updateDraft((c) => ({ ...c, pipes: relabel(c.pipes) }))}>
            relabel P1…Pn
          </button>
          <button type="button" onClick={() => load({ ...emptyDraft(), name: draft.name })}>
            clear the board
          </button>
          <button type="button" onClick={resetDraft}>
            start over
          </button>
        </div>
      </section>

      <div className="columns">
        <section className="panel">
          <header className="panel__header">
            <div>
              <h3>Start from something</h3>
              <p className="panel__hint">A shipped level, a random solvable board, or your own JSON.</p>
            </div>
          </header>

          <div className="designer__samples">
            {SAMPLES.map((sample) => (
              <button key={sample.name} type="button" onClick={() => load(structuredClone(sample))}>
                {sample.name}
              </button>
            ))}
          </div>

          <div className="generator">
            <label>
              <span>colors</span>
              <input
                type="number"
                min={1}
                max={COLOR_NAMES.length}
                value={generator.colors}
                onChange={(event) =>
                  setGenerator((current) => ({ ...current, colors: Number(event.target.value) }))
                }
              />
            </label>
            <label>
              <span>empty pipes</span>
              <input
                type="number"
                min={1}
                max={6}
                value={generator.emptyPipes}
                onChange={(event) =>
                  setGenerator((current) => ({ ...current, emptyPipes: Number(event.target.value) }))
                }
              />
            </label>
            <label>
              <span>shuffles</span>
              <input
                type="number"
                min={1}
                max={400}
                value={generator.shuffles}
                onChange={(event) =>
                  setGenerator((current) => ({ ...current, shuffles: Number(event.target.value) }))
                }
              />
            </label>
            <button
              type="button"
              onClick={() =>
                load({
                  name: draft.name || 'Generated board',
                  description: `${generator.colors} colors, ${generator.emptyPipes} spare pipes`,
                  pipes: generateBoard(generator),
                })
              }
            >
              🎲 generate a solvable board
            </button>
          </div>

          <div className="import">
            <textarea
              value={importText}
              placeholder='{"pipes": {"P1": ["Blue", "Red"], "P2": []}}'
              rows={5}
              onChange={(event) => setImportText(event.target.value)}
            />
            <div className="import__actions">
              <button type="button" disabled={!importText.trim()} onClick={() => importJson(importText)}>
                load this JSON
              </button>
              <button type="button" onClick={() => fileInput.current?.click()}>
                open a file…
              </button>
              <input
                ref={fileInput}
                type="file"
                accept="application/json,.json"
                hidden
                onChange={(event) => void openFile(event.target.files?.[0])}
              />
            </div>
            {importError ? <ErrorNotice error={new Error(importError)} onDismiss={() => setImportError(null)} /> : null}
          </div>
        </section>

        <section className="panel">
          <header className="panel__header">
            <div>
              <h3>Save it for later</h3>
              <p className="panel__hint">
                A saved scenario can be replayed, solved and listed under <code>Scenarios</code>.
              </p>
            </div>
          </header>

          <div className="save">
            <input
              value={saveName}
              placeholder={draft.name || 'scenario name'}
              onChange={(event) => setSaveName(event.target.value)}
            />
            <button
              type="button"
              disabled={blocked || createScenario.isPending}
              onClick={save}
            >
              {createScenario.isPending ? 'saving…' : 'save as scenario'}
            </button>
          </div>

          {createScenario.isSuccess ? (
            <div className="notice notice--ok">
              saved as <strong>{createScenario.data.name}</strong>{' '}
              <code>{createScenario.data.id}</code>
            </div>
          ) : null}
          <ErrorNotice error={createScenario.error} onDismiss={() => createScenario.reset()} />
        </section>
      </div>

      <div className="columns">
        <JsonPanel
          title="Scenario payload"
          hint="POST /api/v1/scenarios"
          value={toScenarioPayload(draft)}
          fileName={`${(draft.name || 'scenario').replace(/\s+/g, '-').toLowerCase()}.json`}
          curl={`curl -sX POST ${origin}/api/v1/scenarios \\\n  -H 'content-type: application/json' \\\n  -d '${JSON.stringify(toScenarioPayload(draft))}'`}
        />
        <JsonPanel
          title="Solve payload"
          hint="POST /api/v1/puzzles/solve — nothing is stored"
          value={toSolvePayload(draft)}
          fileName="solve-request.json"
          curl={`curl -sX POST ${origin}/api/v1/puzzles/solve \\\n  -H 'content-type: application/json' \\\n  -d '${JSON.stringify(toSolvePayload(draft))}'`}
        />
        <JsonPanel
          title="Level file"
          hint="the historical shape, accepted by the CLI and the API alike"
          value={toLevelDocument(draft)}
          fileName={`${(draft.name || 'level').replace(/\s+/g, '-').toLowerCase()}.json`}
        />
      </div>
    </div>
  )
}
