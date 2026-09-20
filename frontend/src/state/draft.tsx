/**
 * The scenario currently being designed.
 *
 * It is the thread of the whole app: the designer writes it, the solver page
 * reads it, and starting a game hands it to the API. It survives a reload in
 * `localStorage`, so a board is never lost to a refresh.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'

import { type Draft, emptyDraft } from '../game/scenario'

const STORAGE_KEY = 'water-sort:draft'

interface DraftContextValue {
  draft: Draft
  setDraft: (draft: Draft) => void
  updateDraft: (change: (draft: Draft) => Draft) => void
  resetDraft: () => void
}

const DraftContext = createContext<DraftContextValue | null>(null)

function readStoredDraft(): Draft {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return emptyDraft()
    const parsed = JSON.parse(raw) as Draft
    if (!parsed || !Array.isArray(parsed.pipes)) return emptyDraft()
    return {
      name: typeof parsed.name === 'string' ? parsed.name : '',
      description: typeof parsed.description === 'string' ? parsed.description : '',
      pipes: parsed.pipes.map((pipe, index) => ({
        label: typeof pipe?.label === 'string' ? pipe.label : `P${index + 1}`,
        colors: Array.isArray(pipe?.colors) ? pipe.colors.filter((c) => typeof c === 'string') : [],
      })),
    }
  } catch {
    return emptyDraft()
  }
}

export function DraftProvider({ children }: { children: ReactNode }) {
  const [draft, setDraft] = useState<Draft>(readStoredDraft)

  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(draft))
    } catch {
      // A full or disabled storage is not worth breaking the designer over.
    }
  }, [draft])

  const updateDraft = useCallback(
    (change: (current: Draft) => Draft) => setDraft((current) => change(current)),
    [],
  )

  const resetDraft = useCallback(() => setDraft(emptyDraft()), [])

  const value = useMemo(
    () => ({ draft, setDraft, updateDraft, resetDraft }),
    [draft, updateDraft, resetDraft],
  )

  return <DraftContext.Provider value={value}>{children}</DraftContext.Provider>
}

export function useDraft(): DraftContextValue {
  const value = useContext(DraftContext)
  if (!value) throw new Error('useDraft must be used inside a <DraftProvider>')
  return value
}
