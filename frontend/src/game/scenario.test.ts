/** Reading a board in, and handing it back to the API. */

import { describe, expect, it } from 'vitest'

import { draftFromJson, toBoardDocument, toLevelDocument, validate } from './scenario'

describe('importing a board', () => {
  it('reads the historical level file shape', () => {
    const draft = draftFromJson('{"pipes": {"P1": ["Red", "Blue"], "P2": []}}')

    expect(draft.pipes).toEqual([
      { label: 'P1', colors: ['Red', 'Blue'] },
      { label: 'P2', colors: [] },
    ])
  })

  it('reads the list shape, a scenario payload and a scenario answer alike', () => {
    expect(draftFromJson('{"pipes": [{"label": "A", "colors": ["Red"]}, {"colors": []}]}').pipes)
      .toEqual([
        { label: 'A', colors: ['Red'] },
        { label: 'P2', colors: [] },
      ])

    const payload = draftFromJson(
      '{"name": "Level 1", "puzzle": {"pipes": {"P1": ["Red"], "P2": []}}}',
    )
    expect(payload.name).toBe('Level 1')
    expect(payload.pipes).toHaveLength(2)

    const answer = draftFromJson(
      '{"id": "x", "name": "Saved", "pipes": [{"id": "a", "label": "P1", "colors": ["Red"]}]}',
    )
    expect(answer.name).toBe('Saved')
    expect(answer.pipes[0].colors).toEqual(['Red'])
  })

  it('refuses what it cannot read', () => {
    expect(() => draftFromJson('nope')).toThrow(/not valid JSON/)
    expect(() => draftFromJson('{"nothing": 1}')).toThrow(/no board found/)
    expect(() => draftFromJson('{"pipes": {"P1": "Red"}}')).toThrow(/list of colors/)
  })
})

describe('exporting a board', () => {
  const draft = {
    name: ' Level 9 ',
    description: '',
    pipes: [
      { label: 'P1', colors: ['red', 'light_blue'] },
      { label: 'P2', colors: [] },
    ],
  }

  it('canonicalises the color names the API expects', () => {
    expect(toBoardDocument(draft)).toEqual({
      pipes: [
        { label: 'P1', colors: ['Red', 'LightBlue'] },
        { label: 'P2', colors: [] },
      ],
    })

    expect(toLevelDocument(draft)).toEqual({ pipes: { P1: ['Red', 'LightBlue'], P2: [] } })
  })
})

describe('validating a board', () => {
  const problems = (pipes: Array<{ label: string; colors: string[] }>) =>
    validate({ name: '', description: '', pipes }).map((problem) => problem.message)

  it('catches what the API would refuse', () => {
    expect(problems([{ label: 'P1', colors: ['Red'] }])).toContain(
      'a puzzle needs at least 2 pipes',
    )
    expect(
      problems([
        { label: 'P1', colors: ['Red', 'Red', 'Red', 'Red', 'Red'] },
        { label: 'P2', colors: [] },
      ]),
    ).toContain("pipe 'P1' holds 5 units, the maximum is 4")
    expect(
      problems([
        { label: 'P1', colors: ['Chartreuse'] },
        { label: 'P2', colors: [] },
      ]),
    ).toContain("pipe 'P1' uses an unknown color 'Chartreuse'")
    expect(
      problems([
        { label: 'P1', colors: [] },
        { label: 'P1', colors: [] },
      ]),
    ).toContain("duplicated pipe label 'P1'")
  })

  it('warns about a board no solver could finish', () => {
    expect(
      problems([
        { label: 'P1', colors: ['Red', 'Red', 'Red'] },
        { label: 'P2', colors: [] },
      ]),
    ).toContain('Red appears 3 times, which is not a multiple of 4: the board cannot be solved')
  })
})
