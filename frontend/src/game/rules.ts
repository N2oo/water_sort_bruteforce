/**
 * The pour rules, mirrored from `crates/core/src/pipe.rs`.
 *
 * The API stays the authority: every move a player makes is posted to it and
 * the board it answers with is the one rendered. These functions only serve the
 * things that must not need a round trip — highlighting the pipes a selected
 * pipe may pour into, and stepping through a solution locally.
 */

export const PIPE_CAPACITY = 4

/** A pipe reduced to what the rules care about: its content, bottom first. */
export type Column = readonly string[]

export interface BoardState {
  /** Pipe contents, in board order. */
  columns: string[][]
}

const isEmpty = (pipe: Column) => pipe.length === 0
const isFilled = (pipe: Column) => pipe.length >= PIPE_CAPACITY
const top = (pipe: Column) => (pipe.length ? pipe[pipe.length - 1] : undefined)

/** `Pipe::all_same_colors`: at least one unit, and all of them equal. */
export function allSameColors(pipe: Column): boolean {
  return pipe.length >= 1 && pipe.every((color) => color === pipe[0])
}

/** `Pipe::is_completed`: empty, or four units of a single color. */
export function isCompleted(pipe: Column): boolean {
  if (isEmpty(pipe)) return true
  if (pipe.length !== PIPE_CAPACITY) return false
  return pipe.every((color) => color === pipe[0])
}

/**
 * `Pipe::can_pour`, unit by unit. Note the rule that surprises everyone: an
 * already uniform pipe may not be emptied into an empty one, because that move
 * never brings the board closer to a solution.
 */
export function canPourOnce(from: Column, to: Column): boolean {
  if (from === to) return false
  if (isEmpty(from) && isEmpty(to)) return false
  if (allSameColors(from) && isEmpty(to)) return false
  if (isEmpty(to) && !isEmpty(from)) return true
  if (isFilled(to) || isEmpty(from)) return false
  return top(from) === top(to)
}

/** Whether a whole move is legal, i.e. at least one unit would flow. */
export function canPour(from: Column, to: Column): boolean {
  return canPourOnce(from, to)
}

/** `Pipe::pour_into`: keep moving one unit while the rules still allow it. */
export function pour(from: Column, to: Column): { from: string[]; to: string[]; poured: number } {
  const source = [...from]
  const destination = [...to]
  let poured = 0

  while (canPourOnce(source, destination)) {
    destination.push(source.pop() as string)
    poured += 1
  }

  return { from: source, to: destination, poured }
}

/** Apply a move to a board, by pipe index. Returns `null` when it is illegal. */
export function applyMove(board: string[][], fromIndex: number, toIndex: number): string[][] | null {
  if (fromIndex === toIndex) return null
  const source = board[fromIndex]
  const destination = board[toIndex]
  if (!source || !destination || !canPour(source, destination)) return null

  const poured = pour(source, destination)
  const next = board.map((pipe) => [...pipe])
  next[fromIndex] = poured.from
  next[toIndex] = poured.to
  return next
}

/** `Puzzle::completed_pipes`: every settled pipe, an empty one included. */
export function completedPipes(board: string[][]): number {
  return board.filter(isCompleted).length
}

/** The pipes that hold four units of one color — what a player calls "done". */
export function finishedPipes(board: string[][]): number {
  return board.filter((pipe) => pipe.length === PIPE_CAPACITY && isCompleted(pipe)).length
}

/** A board is solved when no pipe holds a mix of colors. */
export function isSolved(board: string[][]): boolean {
  return board.every((pipe) => isCompleted(pipe))
}

/** Every pipe the selected one may legally pour into, by index. */
export function legalTargets(board: string[][], fromIndex: number): number[] {
  const targets: number[] = []
  board.forEach((pipe, index) => {
    if (index !== fromIndex && canPour(board[fromIndex], pipe)) targets.push(index)
  })
  return targets
}

/** True when no move at all is left — the board is stuck, not solved. */
export function isDeadEnd(board: string[][]): boolean {
  if (isSolved(board)) return false
  return board.every((_, index) => legalTargets(board, index).length === 0)
}
