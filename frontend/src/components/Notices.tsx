/** Small, shared pieces of chrome: errors, warnings, empty states, spinners. */

import type { ReactNode } from 'react'

import { ApiError } from '../api/client'
import type { Problem } from '../game/scenario'

/** Turns anything thrown by the API layer into a sentence a player can act on. */
export function explain(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.isUnreachable) {
      return 'The API cannot be reached. Is `cargo run -p water-sort-api` up on :8080?'
    }
    switch (error.code) {
      case 'solver_timeout':
        return `${error.message} — raise SOLVER_TIMEOUT_SECONDS, or simplify the board.`
      default:
        return error.message
    }
  }
  if (error instanceof Error) return error.message
  return String(error)
}

export function ErrorNotice({ error, onDismiss }: { error: unknown; onDismiss?: () => void }) {
  if (!error) return null
  const code = error instanceof ApiError ? error.code : 'error'

  return (
    <div className="notice notice--error" role="alert">
      <span className="notice__code">{code}</span>
      <span className="notice__message">{explain(error)}</span>
      {onDismiss ? (
        <button type="button" className="notice__dismiss" onClick={onDismiss} aria-label="dismiss">
          ×
        </button>
      ) : null}
    </div>
  )
}

export function ProblemList({ problems }: { problems: Problem[] }) {
  if (!problems.length) return null

  return (
    <ul className="problems">
      {problems.map((problem, index) => (
        <li key={index} className={`problems__item problems__item--${problem.severity}`}>
          {problem.message}
        </li>
      ))}
    </ul>
  )
}

export function EmptyState({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="empty">
      <h3>{title}</h3>
      {children}
    </div>
  )
}

export function Spinner({ label = 'working…' }: { label?: string }) {
  return (
    <span className="spinner" role="status">
      <span className="spinner__dot" />
      {label}
    </span>
  )
}
