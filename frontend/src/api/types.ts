/**
 * The wire types of `water-sort-api`, mirrored one for one from
 * `crates/api/src/adapters/inbound/http/dto.rs`.
 */

export type Uuid = string

/** A pipe as the API hands it out: a uuid, a readable label, and its content. */
export interface Pipe {
  id: Uuid
  label: string
  /** Bottom first, at most `PIPE_CAPACITY` entries. */
  colors: string[]
}

export interface Scenario {
  id: Uuid
  name: string
  description: string | null
  pipes: Pipe[]
  created_at: string
}

export interface PlayedMove {
  id: Uuid
  sequence: number
  from: Uuid
  to: Uuid
  from_label: string | null
  to_label: string | null
  notation: string
  played_at: string
  /** Only `GET /games/{id}/moves` fills this in. */
  board_after?: Pipe[]
}

export interface Game {
  id: Uuid
  scenario_id: Uuid | null
  name: string | null
  status: string
  solved: boolean
  moves_played: number
  completed_pipes: number
  pipes: Pipe[]
  initial_pipes: Pipe[]
  moves: PlayedMove[]
  created_at: string
  updated_at: string
}

export interface PlayedMovesResponse {
  played: PlayedMove[]
  game: Game
}

export interface RollBackResponse {
  rolled_back: PlayedMove[]
  game: Game
}

export interface SolutionMove {
  from: Uuid
  to: Uuid
  from_label: string
  to_label: string
  notation: string
}

export interface Solution {
  solved: boolean
  moves_count: number
  moves: SolutionMove[]
}

/**
 * A board as it is *submitted*: the two shapes `water-sort-format` accepts.
 * The designer always produces the list form, which keeps the pipe order.
 */
export interface BoardDocument {
  pipes: Array<{ id?: Uuid; label?: string; colors: string[] }> | Record<string, string[]>
}

export interface CreateScenarioRequest {
  name: string
  description?: string | null
  puzzle: BoardDocument
}

export interface CreateGameRequest {
  scenario_id?: Uuid
  puzzle?: BoardDocument
  name?: string
  save_as_scenario?: string
}

export interface MoveRequest {
  from: Uuid
  to: Uuid
}

export interface Page {
  limit?: number
  offset?: number
}
