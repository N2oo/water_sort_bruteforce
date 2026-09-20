/**
 * The TypeScript rules must agree with `crates/core`. The cases below are the
 * ones the Rust tests pin down, replayed here.
 */

import { describe, expect, it } from 'vitest'

import {
  allSameColors,
  applyMove,
  canPour,
  completedPipes,
  isCompleted,
  isDeadEnd,
  isSolved,
  legalTargets,
  pour,
} from './rules'

describe('a single pipe', () => {
  it('is uniform only when it holds at least one unit, all alike', () => {
    expect(allSameColors([])).toBe(false)
    expect(allSameColors(['Blue'])).toBe(true)
    expect(allSameColors(['Blue', 'Blue'])).toBe(true)
    expect(allSameColors(['Blue', 'Blue', 'Red', 'Blue'])).toBe(false)
  })

  it('is completed when empty, or four units of one color', () => {
    expect(isCompleted([])).toBe(true)
    expect(isCompleted(['Red', 'Red', 'Red', 'Red'])).toBe(true)
    expect(isCompleted(['Red', 'Red', 'Red'])).toBe(false)
    expect(isCompleted(['Red', 'Red', 'Red', 'Blue'])).toBe(false)
  })
})

describe('pouring', () => {
  it('accepts a mixed pipe poured into an empty one', () => {
    expect(canPour(['Red', 'Blue'], [])).toBe(true)
  })

  it('refuses a uniform pipe poured into an empty one', () => {
    expect(canPour(['Red'], [])).toBe(false)
    expect(canPour(['Red', 'Red'], [])).toBe(false)
  })

  it('refuses two empty pipes, a full destination and mismatched tops', () => {
    expect(canPour([], [])).toBe(false)
    expect(canPour(['Red'], ['Blue', 'Blue', 'Blue', 'Blue'])).toBe(false)
    expect(canPour(['Red', 'Blue'], ['Red'])).toBe(false)
  })

  it('moves the whole top run, capped by the room left', () => {
    expect(pour(['Blue', 'Red', 'Red'], ['Red'])).toEqual({
      from: ['Blue'],
      to: ['Red', 'Red', 'Red'],
      poured: 2,
    })

    expect(pour(['Red', 'Red', 'Red'], ['Blue', 'Red', 'Red'])).toEqual({
      from: ['Red', 'Red'],
      to: ['Blue', 'Red', 'Red', 'Red'],
      poured: 1,
    })
  })
})

describe('a board', () => {
  const board = [
    ['Red', 'Blue'],
    ['Blue', 'Red'],
    [],
  ]

  it('reports the moves a pipe may play', () => {
    expect(legalTargets(board, 0)).toEqual([2])
    expect(legalTargets(board, 2)).toEqual([])
  })

  it('applies a move, or refuses it', () => {
    expect(applyMove(board, 0, 2)).toEqual([['Red'], ['Blue', 'Red'], ['Blue']])
    expect(applyMove(board, 0, 1)).toBeNull()
    expect(applyMove(board, 0, 0)).toBeNull()
  })

  it('counts settled pipes the way the API does — empty ones included', () => {
    expect(completedPipes([['Red', 'Red', 'Red', 'Red'], [], ['Red', 'Blue']])).toBe(2)
  })

  it('knows a solved board from a stuck one', () => {
    expect(isSolved([['Red', 'Red', 'Red', 'Red'], []])).toBe(true)
    expect(isSolved(board)).toBe(false)
    expect(isDeadEnd(board)).toBe(false)
    // Two uniform pipes and one empty: no legal pour is left.
    expect(isDeadEnd([['Red', 'Red'], ['Blue', 'Blue'], []])).toBe(true)
  })
})
