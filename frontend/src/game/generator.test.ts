/**
 * The generator promises solvable boards. A depth first search — the same idea
 * as `crates/core/src/solver.rs`, written here only to check that promise —
 * finds a solution for each of them.
 */

import { describe, expect, it } from 'vitest'

import { generateBoard } from './generator'
import { applyMove, isSolved, legalTargets } from './rules'

function solvable(board: string[][], seen = new Set<string>()): boolean {
  if (isSolved(board)) return true

  const key = board.map((pipe) => pipe.join(',')).join('|')
  if (seen.has(key)) return false
  seen.add(key)
  if (seen.size > 200_000) throw new Error('search space too large for this test')

  for (let from = 0; from < board.length; from += 1) {
    for (const to of legalTargets(board, from)) {
      const next = applyMove(board, from, to)
      if (next && solvable(next, seen)) return true
    }
  }

  return false
}

describe('the board generator', () => {
  it('always hands out a solvable board', () => {
    for (let seed = 1; seed <= 12; seed += 1) {
      const pipes = generateBoard({ colors: 4, emptyPipes: 2, shuffles: 30, seed })
      const board = pipes.map((pipe) => pipe.colors)

      expect(board.flat().length).toBe(16)
      expect(solvable(board)).toBe(true)
    }
  })

  it('is deterministic for a given seed, and scrambles the solved board', () => {
    const first = generateBoard({ colors: 5, emptyPipes: 2, shuffles: 40, seed: 7 })
    const second = generateBoard({ colors: 5, emptyPipes: 2, shuffles: 40, seed: 7 })

    expect(first).toEqual(second)
    expect(isSolved(first.map((pipe) => pipe.colors))).toBe(false)
  })

  it('never overfills a pipe', () => {
    const pipes = generateBoard({ colors: 6, emptyPipes: 3, shuffles: 80, seed: 3 })
    expect(pipes.every((pipe) => pipe.colors.length <= 4)).toBe(true)
  })
})
