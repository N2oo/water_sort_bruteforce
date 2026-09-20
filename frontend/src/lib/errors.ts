/** Turns anything thrown by the API layer into a sentence a player can act on. */

import { ApiError } from '@/api/client'

export function explain(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.isUnreachable) {
      return 'The API cannot be reached. Is `cargo run -p water-sort-api` up on :8080?'
    }
    if (error.isTimeout) {
      return `${error.message} — raise SOLVER_TIMEOUT_SECONDS, or simplify the board.`
    }
    return error.message
  }
  if (error instanceof Error) return error.message
  return String(error)
}

/** The `code` of the API's error envelope, for the alert's title. */
export function errorCode(error: unknown): string {
  return error instanceof ApiError ? error.code : 'error'
}
