/**
 * The one place that talks HTTP.
 *
 * Every failure comes back as an {@link ApiError} carrying the API's own
 * `{ error: { code, message } }` envelope, so a component can react to
 * `illegal_move` or `solver_timeout` without parsing strings.
 */

/** Empty by default: the dev server proxies `/api` and `/health` to the API. */
export const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL ?? '').replace(/\/$/, '')

export class ApiError extends Error {
  readonly status: number
  readonly code: string

  constructor(status: number, code: string, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.code = code
  }

  /** `true` when the API could not be reached at all. */
  get isUnreachable(): boolean {
    return this.code === 'network_error' || this.code === 'api_unavailable'
  }

  /** `true` when the board or the move was refused by the rules, not by us. */
  get isRuleViolation(): boolean {
    return ['illegal_move', 'unknown_pipe', 'same_pipe', 'invalid_board'].includes(this.code)
  }

  get isTimeout(): boolean {
    return this.code === 'solver_timeout'
  }
}

interface RequestOptions {
  method?: 'GET' | 'POST' | 'DELETE'
  body?: unknown
  query?: Record<string, string | number | undefined>
  signal?: AbortSignal
}

function withQuery(path: string, query?: RequestOptions['query']): string {
  if (!query) return path
  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined) search.set(key, String(value))
  }
  const rendered = search.toString()
  return rendered ? `${path}?${rendered}` : path
}

export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = 'GET', body, query, signal } = options

  let response: Response
  try {
    response = await fetch(`${API_BASE_URL}${withQuery(path, query)}`, {
      method,
      signal,
      headers: body === undefined ? undefined : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch (cause) {
    if (cause instanceof DOMException && cause.name === 'AbortError') throw cause
    throw new ApiError(0, 'network_error', `the API could not be reached at ${API_BASE_URL || window.location.origin}`)
  }

  if (response.status === 204) return undefined as T

  const raw = await response.text()
  const payload: unknown = raw ? safeParse(raw) : null

  if (!response.ok) {
    // The API always answers its own envelope. Anything else — an HTML error
    // page from a proxy, an empty body — means the call never reached it.
    const envelope =
      payload && typeof payload === 'object'
        ? (payload as { error?: { code?: string; message?: string } })
        : null
    const fallbackCode = response.status >= 500 ? 'api_unavailable' : 'unexpected_error'
    const fallbackMessage =
      response.status >= 500
        ? `the API answered ${response.status} — is it running behind ${API_BASE_URL || 'this origin'}?`
        : `the API answered ${response.status} ${response.statusText}`.trim()

    throw new ApiError(
      response.status,
      envelope?.error?.code ?? fallbackCode,
      envelope?.error?.message || fallbackMessage,
    )
  }

  return payload as T
}

function safeParse(raw: string): unknown {
  try {
    return JSON.parse(raw)
  } catch {
    return raw
  }
}
