/**
 * The JSON payload of whatever is on screen, ready to be curl'd at the API.
 */

import { useState } from 'react'

interface JsonPanelProps {
  title: string
  /** What the payload is for, e.g. the endpoint that accepts it. */
  hint?: string
  value: unknown
  /** File name proposed when the payload is downloaded. */
  fileName?: string
  /** A ready to paste `curl` invocation for this payload. */
  curl?: string
}

export function JsonPanel({ title, hint, value, fileName = 'scenario.json', curl }: JsonPanelProps) {
  const [copied, setCopied] = useState<'json' | 'curl' | null>(null)
  const text = JSON.stringify(value, null, 2)

  const copy = async (what: 'json' | 'curl', payload: string) => {
    try {
      await navigator.clipboard.writeText(payload)
      setCopied(what)
      window.setTimeout(() => setCopied(null), 1500)
    } catch {
      // Clipboard access can be refused; the text is on screen either way.
    }
  }

  const download = () => {
    const blob = new Blob([text], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = fileName
    link.click()
    URL.revokeObjectURL(url)
  }

  return (
    <section className="panel json-panel">
      <header className="panel__header">
        <div>
          <h3>{title}</h3>
          {hint ? <p className="panel__hint">{hint}</p> : null}
        </div>
        <div className="panel__actions">
          <button type="button" onClick={() => void copy('json', text)}>
            {copied === 'json' ? 'copied' : 'copy JSON'}
          </button>
          <button type="button" onClick={download}>
            download
          </button>
          {curl ? (
            <button type="button" onClick={() => void copy('curl', curl)}>
              {copied === 'curl' ? 'copied' : 'copy curl'}
            </button>
          ) : null}
        </div>
      </header>
      <pre className="json-panel__code">{text}</pre>
    </section>
  )
}
