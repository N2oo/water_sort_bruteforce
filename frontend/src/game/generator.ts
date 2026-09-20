/**
 * A random board that is **guaranteed solvable**.
 *
 * It starts from the solved board and undoes pours. A backward move is only
 * taken when the forward pour that cancels it is legal and moves exactly the
 * same units — `Pipe::can_pour` pours greedily, so the units moved back have to
 * be the whole top run of the source, and the destination must not be left
 * uniform-into-empty, which the rules forbid. Every board it hands out is
 * therefore one legal pour sequence away from being solved.
 */

import { COLOR_NAMES } from './colors'
import { PIPE_CAPACITY, isSolved } from './rules'
import type { DraftPipe } from './scenario'

export interface GeneratorOptions {
  /** How many colors, one full pipe each. */
  colors: number
  /** Spare pipes, empty on the solved board. */
  emptyPipes: number
  /** How many pours to undo; more means a longer solution. */
  shuffles: number
  /** Same seed, same board. */
  seed?: number
}

export const DEFAULT_GENERATOR: GeneratorOptions = { colors: 5, emptyPipes: 2, shuffles: 40 }

/** A tiny xorshift, so a seed always yields the same board. */
function randomSource(seed: number): () => number {
  let state = seed >>> 0 || 0x9e3779b9
  return () => {
    state ^= state << 13
    state ^= state >>> 17
    state ^= state << 5
    return (state >>> 0) / 0x100000000
  }
}

const pick = <T,>(items: T[], random: () => number): T =>
  items[Math.floor(random() * items.length)]

/** Length of the run of identical units at the top of a pipe. */
function topRun(pipe: string[]): number {
  if (!pipe.length) return 0
  const color = pipe[pipe.length - 1]
  let run = 0
  while (run < pipe.length && pipe[pipe.length - 1 - run] === color) run += 1
  return run
}

/**
 * Undo one pour: move `units` of the top color from `source` onto `target`,
 * which is where they would have been poured from.
 */
function undoOnePour(board: string[][], random: () => number): boolean {
  const donors = board.map((_, index) => index).filter((index) => board[index].length > 0)
  if (!donors.length) return false

  for (const from of shuffled(donors, random)) {
    const run = topRun(board[from])
    const color = board[from][board[from].length - 1]

    const receivers = board
      .map((_, index) => index)
      .filter((index) => {
        if (index === from) return false
        const pipe = board[index]
        if (pipe.length >= PIPE_CAPACITY) return false
        // The forward pour must move the whole top run of this pipe, nothing
        // that was already sitting there.
        return pipe.length === 0 || pipe[pipe.length - 1] !== color
      })

    for (const to of shuffled(receivers, random)) {
      const room = PIPE_CAPACITY - board[to].length
      const candidates: number[] = []
      for (let units = 1; units <= Math.min(run, room); units += 1) {
        const emptiesDonor = units === board[from].length
        // Pouring a uniform pipe into an empty one is refused by the rules, so
        // that backward move could never be undone.
        if (emptiesDonor && board[to].length === 0) continue
        // Taking the whole run without emptying the donor would leave another
        // color on top of it, and the forward pour would no longer be legal.
        if (!emptiesDonor && units === run) continue
        candidates.push(units)
      }
      if (!candidates.length) continue

      const units = pick(candidates, random)
      for (let unit = 0; unit < units; unit += 1) board[to].push(board[from].pop() as string)
      return true
    }
  }

  return false
}

function shuffled<T>(items: T[], random: () => number): T[] {
  const copy = [...items]
  for (let index = copy.length - 1; index > 0; index -= 1) {
    const other = Math.floor(random() * (index + 1))
    ;[copy[index], copy[other]] = [copy[other], copy[index]]
  }
  return copy
}

export function generateBoard(options: GeneratorOptions): DraftPipe[] {
  const colors = Math.max(1, Math.min(options.colors, COLOR_NAMES.length))
  const empties = Math.max(1, options.emptyPipes)
  const random = randomSource(options.seed ?? Math.floor(Math.random() * 0xffffffff))

  const board: string[][] = [
    ...Array.from({ length: colors }, (_, index) =>
      Array.from({ length: PIPE_CAPACITY }, () => COLOR_NAMES[index] as string),
    ),
    ...Array.from({ length: empties }, () => [] as string[]),
  ]

  const rounds = Math.max(1, options.shuffles)
  for (let round = 0; round < rounds; round += 1) {
    if (!undoOnePour(board, random)) break
  }

  // A board that came back to the solved state is no puzzle at all.
  if (isSolved(board)) undoOnePour(board, random)

  return board.map((pipe, index) => ({ label: `P${index + 1}`, colors: pipe }))
}
