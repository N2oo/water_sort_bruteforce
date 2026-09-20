/** Every route of the API, one function each. No React in here. */

import { request } from './client'
import type {
  CreateGameRequest,
  CreateScenarioRequest,
  Game,
  MoveRequest,
  Page,
  PlayedMove,
  PlayedMovesResponse,
  RollBackResponse,
  Scenario,
  Solution,
  Uuid,
} from './types'

const page = (paging?: Page) => ({ limit: paging?.limit, offset: paging?.offset })

export const health = () => request<{ status: string }>('/health')

export const listScenarios = (paging?: Page) =>
  request<Scenario[]>('/api/v1/scenarios', { query: page(paging) })

export const getScenario = (id: Uuid) => request<Scenario>(`/api/v1/scenarios/${id}`)

export const createScenario = (body: CreateScenarioRequest) =>
  request<Scenario>('/api/v1/scenarios', { method: 'POST', body })

export const deleteScenario = (id: Uuid) =>
  request<void>(`/api/v1/scenarios/${id}`, { method: 'DELETE' })

export const solveScenario = (id: Uuid) =>
  request<Solution>(`/api/v1/scenarios/${id}/solve`, { method: 'POST' })

/** Solve a board that is never stored. */
export const solvePuzzle = (body: CreateScenarioRequest['puzzle'], signal?: AbortSignal) =>
  request<Solution>('/api/v1/puzzles/solve', { method: 'POST', body: { puzzle: body }, signal })

export const listGames = (paging?: Page) =>
  request<Game[]>('/api/v1/games', { query: page(paging) })

export const getGame = (id: Uuid) => request<Game>(`/api/v1/games/${id}`)

export const createGame = (body: CreateGameRequest) =>
  request<Game>('/api/v1/games', { method: 'POST', body })

export const deleteGame = (id: Uuid) => request<void>(`/api/v1/games/${id}`, { method: 'DELETE' })

/** The history, board snapshots included. */
export const getGameMoves = (id: Uuid) => request<PlayedMove[]>(`/api/v1/games/${id}/moves`)

/** One move or a batch; a batch is refused as a whole if any move is illegal. */
export const playMoves = (id: Uuid, moves: MoveRequest[]) =>
  request<PlayedMovesResponse>(`/api/v1/games/${id}/moves`, {
    method: 'POST',
    body: moves.length === 1 ? moves[0] : { moves },
  })

export const rollBack = (id: Uuid, steps: number) =>
  request<RollBackResponse>(`/api/v1/games/${id}/rollback`, { method: 'POST', body: { steps } })

export const resetGame = (id: Uuid) =>
  request<RollBackResponse>(`/api/v1/games/${id}/reset`, { method: 'POST' })

export const solveGame = (id: Uuid) =>
  request<Solution>(`/api/v1/games/${id}/solve`, { method: 'POST' })
