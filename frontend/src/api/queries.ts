/**
 * TanStack Query bindings.
 *
 * Reads are queries, writes are mutations. Every mutation that changes a game
 * seeds the game cache with the fresh session the API just returned, then
 * invalidates what it cannot know: the move history and the listings.
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import * as api from './endpoints'
import type { ApiError } from './client'
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

export const queryKeys = {
  health: ['health'] as const,
  scenarios: (paging?: Page) => ['scenarios', paging ?? {}] as const,
  scenario: (id: Uuid) => ['scenarios', id] as const,
  games: (paging?: Page) => ['games', paging ?? {}] as const,
  game: (id: Uuid) => ['games', id] as const,
  gameMoves: (id: Uuid) => ['games', id, 'moves'] as const,
}

/* ------------------------------------------------------------------ reads */

export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: () => api.health(),
    refetchInterval: 30_000,
    retry: false,
    staleTime: 10_000,
  })
}

export function useScenarios(paging?: Page) {
  return useQuery<Scenario[], ApiError>({
    queryKey: queryKeys.scenarios(paging),
    queryFn: () => api.listScenarios(paging),
  })
}

export function useScenario(id: Uuid | undefined) {
  return useQuery<Scenario, ApiError>({
    queryKey: queryKeys.scenario(id ?? 'none'),
    queryFn: () => api.getScenario(id as Uuid),
    enabled: Boolean(id),
  })
}

export function useGames(paging?: Page) {
  return useQuery<Game[], ApiError>({
    queryKey: queryKeys.games(paging),
    queryFn: () => api.listGames(paging),
  })
}

export function useGame(id: Uuid | undefined) {
  return useQuery<Game, ApiError>({
    queryKey: queryKeys.game(id ?? 'none'),
    queryFn: () => api.getGame(id as Uuid),
    enabled: Boolean(id),
  })
}

export function useGameMoves(id: Uuid | undefined) {
  return useQuery<PlayedMove[], ApiError>({
    queryKey: queryKeys.gameMoves(id ?? 'none'),
    queryFn: () => api.getGameMoves(id as Uuid),
    enabled: Boolean(id),
  })
}

/* ----------------------------------------------------------------- writes */

/** Shared by every mutation that hands a fresh `Game` back. */
function useGameWriter<TData extends { game: Game }, TVariables>(
  gameId: Uuid | undefined,
  mutationFn: (variables: TVariables) => Promise<TData>,
) {
  const client = useQueryClient()

  return useMutation<TData, ApiError, TVariables>({
    mutationFn,
    onSuccess: (data) => {
      client.setQueryData(queryKeys.game(data.game.id), data.game)
      if (gameId) void client.invalidateQueries({ queryKey: queryKeys.gameMoves(gameId) })
      void client.invalidateQueries({ queryKey: ['games'] })
    },
  })
}

/** Play one move or a batch; the batch is refused as a whole if one is illegal. */
export function usePlayMoves(gameId: Uuid | undefined) {
  return useGameWriter<PlayedMovesResponse, MoveRequest[]>(gameId, (moves) =>
    api.playMoves(gameId as Uuid, moves),
  )
}

/** Undo the last `steps` moves — they leave the history for good. */
export function useRollBack(gameId: Uuid | undefined) {
  return useGameWriter<RollBackResponse, number>(gameId, (steps) =>
    api.rollBack(gameId as Uuid, steps),
  )
}

export function useResetGame(gameId: Uuid | undefined) {
  return useGameWriter<RollBackResponse, void>(gameId, () => api.resetGame(gameId as Uuid))
}

export function useCreateGame() {
  const client = useQueryClient()

  return useMutation<Game, ApiError, CreateGameRequest>({
    mutationFn: (body) => api.createGame(body),
    onSuccess: (game) => {
      client.setQueryData(queryKeys.game(game.id), game)
      void client.invalidateQueries({ queryKey: ['games'] })
      void client.invalidateQueries({ queryKey: ['scenarios'] })
    },
  })
}

export function useDeleteGame() {
  const client = useQueryClient()

  return useMutation<void, ApiError, Uuid>({
    mutationFn: (id) => api.deleteGame(id),
    onSuccess: (_result, id) => {
      client.removeQueries({ queryKey: queryKeys.game(id) })
      void client.invalidateQueries({ queryKey: ['games'] })
    },
  })
}

export function useCreateScenario() {
  const client = useQueryClient()

  return useMutation<Scenario, ApiError, CreateScenarioRequest>({
    mutationFn: (body) => api.createScenario(body),
    onSuccess: (scenario) => {
      client.setQueryData(queryKeys.scenario(scenario.id), scenario)
      void client.invalidateQueries({ queryKey: ['scenarios'] })
    },
  })
}

export function useDeleteScenario() {
  const client = useQueryClient()

  return useMutation<void, ApiError, Uuid>({
    mutationFn: (id) => api.deleteScenario(id),
    onSuccess: (_result, id) => {
      client.removeQueries({ queryKey: queryKeys.scenario(id) })
      void client.invalidateQueries({ queryKey: ['scenarios'] })
    },
  })
}

/* --------------------------------------------------------------- solving */

/** The three ways to ask for a solution, all long running, all mutations. */

export function useSolvePuzzle() {
  return useMutation<Solution, ApiError, CreateScenarioRequest['puzzle']>({
    mutationFn: (puzzle) => api.solvePuzzle(puzzle),
    retry: false,
  })
}

export function useSolveScenario() {
  return useMutation<Solution, ApiError, Uuid>({
    mutationFn: (id) => api.solveScenario(id),
    retry: false,
  })
}

export function useSolveGame(gameId: Uuid | undefined) {
  return useMutation<Solution, ApiError, void>({
    mutationFn: () => api.solveGame(gameId as Uuid),
    retry: false,
  })
}
