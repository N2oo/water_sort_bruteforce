/**
 * Design a scenario, then choose: play it, or have it solved.
 *
 * Nothing here needs the API. The board is edited locally, validated against
 * the same rules the backend enforces, and shown as the exact JSON payload the
 * API accepts — so a scenario can be taken away and curl'd by hand. The two
 * buttons at the top are the fork the whole app is built around.
 */

import { useMemo, useRef, useState } from 'react'
import {
  AlertTriangle,
  Dices,
  Eraser,
  FileJson,
  FolderOpen,
  Play,
  Plus,
  RotateCcw,
  Save,
  Trash2,
  XCircle,
  Zap,
} from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { API_BASE_URL } from '@/api/client'
import { useCreateGame, useCreateScenario } from '@/api/queries'
import { Board } from '@/components/board'
import { JsonCard } from '@/components/json-card'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
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
import { Checkbox } from '@/components/ui/checkbox'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { Spinner } from '@/components/ui/spinner'
import { Textarea } from '@/components/ui/textarea'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { COLOR_NAMES, colorStyle, type ColorName } from '@/game/colors'
import { DEFAULT_GENERATOR, generateBoard, type GeneratorOptions } from '@/game/generator'
import { PIPE_CAPACITY } from '@/game/rules'
import { SAMPLES } from '@/game/samples'
import { errorCode, explain } from '@/lib/errors'
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
} from '@/game/scenario'
import { useDraft } from '@/state/draft'

type Brush = ColorName | 'erase'

export function DesignerPage() {
  const navigate = useNavigate()
  const { draft, setDraft, updateDraft, resetDraft } = useDraft()

  const [brush, setBrush] = useState<Brush>('Blue')
  const [saveName, setSaveName] = useState('')
  const [saveOnPlay, setSaveOnPlay] = useState(false)
  const [importText, setImportText] = useState('')
  const [generator, setGenerator] = useState<GeneratorOptions>(DEFAULT_GENERATOR)
  const fileInput = useRef<HTMLInputElement>(null)

  const createGame = useCreateGame()
  const createScenario = useCreateScenario()

  const problems = useMemo(() => validate(draft), [draft])
  const blocked = hasErrors(problems)
  const counts = useMemo(() => countByColor(draft), [draft])

  /* ------------------------------------------------------------- editing */

  const paint = (index: number) =>
    updateDraft((current) => ({
      ...current,
      pipes: current.pipes.map((pipe, position) => {
        if (position !== index) return pipe
        if (brush === 'erase') return { ...pipe, colors: pipe.colors.slice(0, -1) }
        if (pipe.colors.length >= PIPE_CAPACITY) return pipe
        return { ...pipe, colors: [...pipe.colors, brush] }
      }),
    }))

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

  const importJson = (text: string) => {
    try {
      setDraft(draftFromJson(text))
      setImportText('')
      toast.success('Board loaded')
    } catch (error) {
      toast.error('That JSON could not be read', {
        description: error instanceof ImportError ? error.message : String(error),
      })
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
      onError: (error) => toast.error(errorCode(error), { description: explain(error) }),
    })
  }

  const save = () => {
    createScenario.mutate(toScenarioPayload({ ...draft, name: saveName.trim() || draft.name }), {
      onSuccess: (scenario) => {
        updateDraft((current) => ({ ...current, name: scenario.name }))
        setSaveName('')
        toast.success(`Saved as “${scenario.name}”`, {
          description: scenario.id,
          action: { label: 'Open', onClick: () => navigate('/scenarios') },
        })
      },
      onError: (error) => toast.error(errorCode(error), { description: explain(error) }),
    })
  }

  const load = (next: Draft) => setDraft(next)
  const origin = API_BASE_URL || 'http://localhost:8080'
  const fileStem = (draft.name || 'scenario').replace(/\s+/g, '-').toLowerCase()

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>1 · Design your scenario</CardTitle>
          <CardDescription>
            Pick a color, click a pipe to pour one unit in. A pipe holds {PIPE_CAPACITY} units,
            bottom first — exactly what the API stores.
          </CardDescription>
          <CardAction className="flex flex-wrap gap-2">
            <Button disabled={blocked || createGame.isPending} onClick={play}>
              {createGame.isPending ? <Spinner /> : <Play />}
              Play this scenario
            </Button>
            <Button variant="secondary" disabled={blocked} onClick={() => navigate('/solve')}>
              <Zap />
              Generate a solution
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent className="flex flex-col gap-6">
          <FieldGroup className="sm:flex-row">
            <Field>
              <FieldLabel htmlFor="scenario-name">Name</FieldLabel>
              <Input
                id="scenario-name"
                value={draft.name}
                placeholder="Level 145"
                onChange={(event) =>
                  updateDraft((current) => ({ ...current, name: event.target.value }))
                }
              />
            </Field>
            <Field>
              <FieldLabel htmlFor="scenario-description">Description</FieldLabel>
              <Input
                id="scenario-description"
                value={draft.description}
                placeholder="optional"
                onChange={(event) =>
                  updateDraft((current) => ({ ...current, description: event.target.value }))
                }
              />
            </Field>
          </FieldGroup>

          <Field>
            <FieldLabel>Palette</FieldLabel>
            <ToggleGroup
              type="single"
              variant="outline"
              value={brush}
              onValueChange={(value) => value && setBrush(value as Brush)}
              className="flex-wrap"
            >
              {COLOR_NAMES.map((color) => {
                const style = colorStyle(color)
                return (
                  <ToggleGroupItem
                    key={color}
                    value={color}
                    aria-label={color}
                    className="data-[state=on]:ring-ring h-8 gap-2 px-3 text-xs font-semibold data-[state=on]:ring-2"
                    style={{ background: style.fill, color: style.ink }}
                  >
                    {color}
                    <span className="opacity-60">{counts.get(color) ?? 0}</span>
                  </ToggleGroupItem>
                )
              })}
              <ToggleGroupItem value="erase" aria-label="erase" className="h-8 gap-2 px-3 text-xs">
                <Eraser />
                Erase
              </ToggleGroupItem>
            </ToggleGroup>
            <FieldDescription>
              Clicking a pipe pours one unit of the selected color; “Erase” removes the top one.
            </FieldDescription>
          </Field>

          <div className="flex flex-wrap items-end gap-3">
            {draft.pipes.map((pipe, index) => (
              <div key={index} className="flex flex-col items-center gap-1">
                <Board pipes={[pipe]} onPipeClick={() => paint(index)} />
                <ButtonGroup>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="icon-xs"
                        onClick={() => clearPipe(index)}
                        aria-label={`empty ${pipe.label}`}
                      >
                        <Eraser />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Empty this pipe</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="icon-xs"
                        onClick={() => removePipe(index)}
                        disabled={draft.pipes.length <= 2}
                        aria-label={`remove ${pipe.label}`}
                      >
                        <Trash2 />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Remove this pipe from the board</TooltipContent>
                  </Tooltip>
                </ButtonGroup>
              </div>
            ))}
            <Button variant="outline" className="border-dashed" onClick={addPipe}>
              <Plus />
              Add a pipe
            </Button>
          </div>

          {problems.map((problem, index) => (
            <Alert key={index} variant={problem.severity === 'error' ? 'destructive' : 'default'}>
              {problem.severity === 'error' ? <XCircle /> : <AlertTriangle />}
              <AlertTitle>{problem.severity === 'error' ? 'The API would refuse this board' : 'This board has no solution'}</AlertTitle>
              <AlertDescription>{problem.message}</AlertDescription>
            </Alert>
          ))}
        </CardContent>

        <CardFooter className="flex-wrap gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => updateDraft((current) => ({ ...current, pipes: relabel(current.pipes) }))}
          >
            Relabel P1…Pn
          </Button>
          <Button variant="outline" size="sm" onClick={() => load({ ...emptyDraft(), name: draft.name })}>
            Clear the board
          </Button>
          <Button variant="outline" size="sm" onClick={resetDraft}>
            <RotateCcw />
            Start over
          </Button>
        </CardFooter>
      </Card>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Start from something</CardTitle>
            <CardDescription>
              A shipped level, a random solvable board, or your own JSON.
            </CardDescription>
          </CardHeader>

          <CardContent className="flex flex-col gap-5">
            <ButtonGroup>
              {SAMPLES.map((sample) => (
                <Button
                  key={sample.name}
                  variant="outline"
                  onClick={() => load(structuredClone(sample))}
                >
                  <FileJson />
                  {sample.name}
                </Button>
              ))}
            </ButtonGroup>

            <Separator />

            <FieldGroup>
              <div className="grid grid-cols-3 gap-3">
                <Field>
                  <FieldLabel htmlFor="generator-colors">Colors</FieldLabel>
                  <Input
                    id="generator-colors"
                    type="number"
                    min={1}
                    max={COLOR_NAMES.length}
                    value={generator.colors}
                    onChange={(event) =>
                      setGenerator((current) => ({ ...current, colors: Number(event.target.value) }))
                    }
                  />
                </Field>
                <Field>
                  <FieldLabel htmlFor="generator-empty">Empty pipes</FieldLabel>
                  <Input
                    id="generator-empty"
                    type="number"
                    min={1}
                    max={6}
                    value={generator.emptyPipes}
                    onChange={(event) =>
                      setGenerator((current) => ({
                        ...current,
                        emptyPipes: Number(event.target.value),
                      }))
                    }
                  />
                </Field>
                <Field>
                  <FieldLabel htmlFor="generator-shuffles">Shuffles</FieldLabel>
                  <Input
                    id="generator-shuffles"
                    type="number"
                    min={1}
                    max={400}
                    value={generator.shuffles}
                    onChange={(event) =>
                      setGenerator((current) => ({
                        ...current,
                        shuffles: Number(event.target.value),
                      }))
                    }
                  />
                </Field>
              </div>
              <Button
                variant="secondary"
                onClick={() =>
                  load({
                    name: draft.name || 'Generated board',
                    description: `${generator.colors} colors, ${generator.emptyPipes} spare pipes`,
                    pipes: generateBoard(generator),
                  })
                }
              >
                <Dices />
                Generate a solvable board
              </Button>
              <FieldDescription>
                Pours are undone from the solved board, so whatever comes out has a solution.
              </FieldDescription>
            </FieldGroup>

            <Separator />

            <Field>
              <FieldLabel htmlFor="import-json">Paste a board</FieldLabel>
              <Textarea
                id="import-json"
                rows={5}
                className="font-mono text-xs"
                value={importText}
                placeholder='{"pipes": {"P1": ["Blue", "Red"], "P2": []}}'
                onChange={(event) => setImportText(event.target.value)}
              />
              <ButtonGroup>
                <Button
                  variant="outline"
                  disabled={!importText.trim()}
                  onClick={() => importJson(importText)}
                >
                  Load this JSON
                </Button>
                <Button variant="outline" onClick={() => fileInput.current?.click()}>
                  <FolderOpen />
                  Open a file…
                </Button>
              </ButtonGroup>
              <input
                ref={fileInput}
                type="file"
                accept="application/json,.json"
                hidden
                onChange={(event) => void openFile(event.target.files?.[0])}
              />
            </Field>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Save it for later</CardTitle>
            <CardDescription>
              A saved scenario can be replayed, solved and listed under Scenarios.
            </CardDescription>
          </CardHeader>

          <CardContent className="flex flex-col gap-4">
            <Field orientation="responsive">
              <Input
                value={saveName}
                placeholder={draft.name || 'scenario name'}
                onChange={(event) => setSaveName(event.target.value)}
                aria-label="scenario name"
              />
              <Button disabled={blocked || createScenario.isPending} onClick={save}>
                {createScenario.isPending ? <Spinner /> : <Save />}
                Save as scenario
              </Button>
            </Field>

            <Field orientation="horizontal">
              <Checkbox
                id="save-on-play"
                checked={saveOnPlay}
                onCheckedChange={(checked) => setSaveOnPlay(checked === true)}
              />
              <FieldLabel htmlFor="save-on-play" className="font-normal">
                Save it as a scenario when I start the game
              </FieldLabel>
            </Field>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 xl:grid-cols-3">
        <JsonCard
          title="Scenario payload"
          description="POST /api/v1/scenarios"
          value={toScenarioPayload(draft)}
          fileName={`${fileStem}.json`}
          curl={`curl -sX POST ${origin}/api/v1/scenarios \\\n  -H 'content-type: application/json' \\\n  -d '${JSON.stringify(toScenarioPayload(draft))}'`}
        />
        <JsonCard
          title="Solve payload"
          description="POST /api/v1/puzzles/solve"
          value={toSolvePayload(draft)}
          fileName="solve-request.json"
          curl={`curl -sX POST ${origin}/api/v1/puzzles/solve \\\n  -H 'content-type: application/json' \\\n  -d '${JSON.stringify(toSolvePayload(draft))}'`}
        />
        <JsonCard
          title="Level file"
          description="level file — the historical shape"
          value={toLevelDocument(draft)}
          fileName={`${fileStem}-level.json`}
        />
      </div>
    </div>
  )
}
