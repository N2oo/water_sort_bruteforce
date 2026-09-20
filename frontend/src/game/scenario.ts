/**
 * The scenario a player is designing, and how it becomes an API payload.
 *
 * A draft is deliberately loose — a half built board is normal — so it is kept
 * as plain labels and color names, validated on the way out and turned into the
 * list form of `water-sort-format`, which preserves the pipe order.
 */

import type { BoardDocument, Pipe } from '../api/types'
import { isKnownColor, normalizeColor } from './colors'
import { PIPE_CAPACITY } from './rules'

export interface DraftPipe {
  label: string
  colors: string[]
}

export interface Draft {
  name: string
  description: string
  pipes: DraftPipe[]
}

export const DEFAULT_PIPE_COUNT = 6

export function emptyPipes(count: number): DraftPipe[] {
  return Array.from({ length: count }, (_, index) => ({ label: `P${index + 1}`, colors: [] }))
}

export function emptyDraft(): Draft {
  return { name: '', description: '', pipes: emptyPipes(DEFAULT_PIPE_COUNT) }
}

export function draftFromPipes(pipes: Pipe[], name = '', description = ''): Draft {
  return {
    name,
    description,
    pipes: pipes.map((pipe, index) => ({
      label: pipe.label || `P${index + 1}`,
      colors: [...pipe.colors],
    })),
  }
}

/** Relabel `P1…Pn` in board order — what the designer does after a removal. */
export function relabel(pipes: DraftPipe[]): DraftPipe[] {
  return pipes.map((pipe, index) => ({ ...pipe, label: `P${index + 1}` }))
}

export interface Problem {
  /** `error` blocks every call to the API; `warning` only makes it unsolvable. */
  severity: 'error' | 'warning'
  message: string
}

/**
 * Everything the API would refuse, checked here so the player is told before a
 * round trip, plus the two warnings the API accepts but the solver cannot act
 * on: a color that does not come in multiples of four, and a full board.
 */
export function validate(draft: Draft): Problem[] {
  const problems: Problem[] = []
  const { pipes } = draft

  if (pipes.length < 2) {
    problems.push({ severity: 'error', message: 'a puzzle needs at least 2 pipes' })
  }

  const labels = new Set<string>()
  for (const pipe of pipes) {
    const label = pipe.label.trim()
    if (!label) {
      problems.push({ severity: 'error', message: 'pipe labels cannot be empty' })
    } else if (labels.has(label)) {
      problems.push({ severity: 'error', message: `duplicated pipe label '${label}'` })
    }
    labels.add(label)

    if (pipe.colors.length > PIPE_CAPACITY) {
      problems.push({
        severity: 'error',
        message: `pipe '${label}' holds ${pipe.colors.length} units, the maximum is ${PIPE_CAPACITY}`,
      })
    }

    for (const color of pipe.colors) {
      if (!isKnownColor(color)) {
        problems.push({
          severity: 'error',
          message: `pipe '${label}' uses an unknown color '${color}'`,
        })
      }
    }
  }

  const units = pipes.reduce((total, pipe) => total + pipe.colors.length, 0)
  if (units === 0) {
    problems.push({ severity: 'error', message: 'the board is empty — pour some colors in' })
  }

  for (const [color, count] of countByColor(draft).entries()) {
    if (count % PIPE_CAPACITY !== 0) {
      problems.push({
        severity: 'warning',
        message: `${color} appears ${count} times, which is not a multiple of ${PIPE_CAPACITY}: the board cannot be solved`,
      })
    }
  }

  const free = pipes.reduce((total, pipe) => total + (PIPE_CAPACITY - pipe.colors.length), 0)
  if (units > 0 && free === 0) {
    problems.push({
      severity: 'warning',
      message: 'every pipe is full: no move can ever be played',
    })
  }

  return problems
}

export function countByColor(draft: Draft): Map<string, number> {
  const counts = new Map<string, number>()
  for (const pipe of draft.pipes) {
    for (const color of pipe.colors) {
      const name = normalizeColor(color) ?? color
      counts.set(name, (counts.get(name) ?? 0) + 1)
    }
  }
  return counts
}

export const hasErrors = (problems: Problem[]) => problems.some((p) => p.severity === 'error')

/** The board as the API takes it: the list form, labels and order preserved. */
export function toBoardDocument(draft: Draft): BoardDocument {
  return {
    pipes: draft.pipes.map((pipe, index) => ({
      label: pipe.label.trim() || `P${index + 1}`,
      colors: pipe.colors.map((color) => normalizeColor(color) ?? color),
    })),
  }
}

/** The historical level file shape, `{"pipes": {"P1": [...]}}`. */
export function toLevelDocument(draft: Draft): BoardDocument {
  const pipes: Record<string, string[]> = {}
  draft.pipes.forEach((pipe, index) => {
    pipes[pipe.label.trim() || `P${index + 1}`] = pipe.colors.map(
      (color) => normalizeColor(color) ?? color,
    )
  })
  return { pipes }
}

/** The body of `POST /api/v1/scenarios`. */
export function toScenarioPayload(draft: Draft) {
  return {
    name: draft.name.trim() || 'Untitled scenario',
    ...(draft.description.trim() ? { description: draft.description.trim() } : {}),
    puzzle: toBoardDocument(draft),
  }
}

/** The body of `POST /api/v1/puzzles/solve`. */
export function toSolvePayload(draft: Draft) {
  return { puzzle: toBoardDocument(draft) }
}

/** The body of `POST /api/v1/games` for a board that is not saved. */
export function toGamePayload(draft: Draft, saveAsScenario?: string) {
  return {
    puzzle: toBoardDocument(draft),
    ...(draft.name.trim() ? { name: draft.name.trim() } : {}),
    ...(saveAsScenario ? { save_as_scenario: saveAsScenario } : {}),
  }
}

export class ImportError extends Error {}

/**
 * Read a draft out of pasted JSON. Everything this app or the API ever prints
 * is accepted: a level file, a board, a scenario payload, a scenario answer and
 * a game answer alike.
 */
export function draftFromJson(raw: string): Draft {
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch (cause) {
    throw new ImportError(`this is not valid JSON: ${(cause as Error).message}`)
  }

  if (!parsed || typeof parsed !== 'object') {
    throw new ImportError('expected a JSON object')
  }

  const document = parsed as Record<string, unknown>
  const name = typeof document.name === 'string' ? document.name : ''
  const description = typeof document.description === 'string' ? document.description : ''

  // `{ name, puzzle: { pipes } }` — a scenario payload, or a solve request.
  const board = (document.puzzle ?? document.board ?? document) as Record<string, unknown>
  const pipes = board.pipes ?? board.initial_pipes

  if (Array.isArray(pipes)) {
    return {
      name,
      description,
      pipes: pipes.map((pipe, index) => readListPipe(pipe, index)),
    }
  }

  if (pipes && typeof pipes === 'object') {
    return {
      name,
      description,
      pipes: Object.entries(pipes as Record<string, unknown>).map(([label, colors]) => ({
        label,
        colors: readColors(colors, label),
      })),
    }
  }

  throw new ImportError("no board found: expected a 'pipes' object or array")
}

function readListPipe(pipe: unknown, index: number): DraftPipe {
  if (!pipe || typeof pipe !== 'object') {
    throw new ImportError(`pipe #${index + 1} is not an object`)
  }
  const entry = pipe as Record<string, unknown>
  const label = typeof entry.label === 'string' ? entry.label : `P${index + 1}`
  return { label, colors: readColors(entry.colors, label) }
}

function readColors(colors: unknown, label: string): string[] {
  if (colors === undefined || colors === null) return []
  if (!Array.isArray(colors)) {
    throw new ImportError(`pipe '${label}' does not hold a list of colors`)
  }
  return colors.map((color) => {
    if (typeof color !== 'string') {
      throw new ImportError(`pipe '${label}' holds a color that is not a string`)
    }
    return color
  })
}
