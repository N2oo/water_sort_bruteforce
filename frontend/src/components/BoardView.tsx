/**
 * A read only board: the shape the API answers with, drawn as a row of pipes.
 *
 * `onPipeClick` turns it into the playable board — the caller decides what a
 * click means, so the same component serves the game, the solution stepper and
 * the scenario previews.
 */

import { PipeView, type PipeRole } from './PipeView'
import { pour } from '../game/rules'

export interface BoardPipe {
  id?: string
  label: string
  colors: string[]
}

interface BoardViewProps {
  pipes: BoardPipe[]
  /** Index of the pipe a pour would start from. */
  selected?: number | null
  /** Indices the selected pipe may pour into. */
  targets?: number[]
  /** Highlighted as the move about to be played, e.g. by the stepper. */
  highlight?: { from: number; to: number } | null
  onPipeClick?: (index: number, pipe: BoardPipe) => void
  disabled?: boolean
  compact?: boolean
}

export function BoardView({
  pipes,
  selected = null,
  targets = [],
  highlight = null,
  onPipeClick,
  disabled = false,
  compact = false,
}: BoardViewProps) {
  const movingUnits =
    selected !== null && pipes[selected]
      ? new Map(
          targets.map((target) => [
            target,
            pour(pipes[selected].colors, pipes[target].colors).poured,
          ]),
        )
      : new Map<number, number>()

  // On a highlighted move, shade the units that are about to leave the source.
  const highlightedUnits =
    highlight && pipes[highlight.from] && pipes[highlight.to]
      ? pour(pipes[highlight.from].colors, pipes[highlight.to].colors).poured
      : 0

  return (
    <div className={`board ${compact ? 'board--compact' : ''}`}>
      {pipes.map((pipe, index) => {
        let role: PipeRole = 'idle'
        if (highlight?.from === index) role = 'source'
        else if (highlight?.to === index) role = 'target'
        else if (selected === index) role = 'selected'
        else if (targets.includes(index)) role = 'target'
        else if (selected !== null) role = 'muted'

        return (
          <PipeView
            key={pipe.id ?? `${pipe.label}-${index}`}
            label={pipe.label}
            colors={pipe.colors}
            role={role}
            moving={highlight && highlight.from === index ? highlightedUnits : 0}
            onClick={onPipeClick ? () => onPipeClick(index, pipe) : undefined}
            disabled={disabled}
            title={
              selected !== null && targets.includes(index)
                ? `pour ${movingUnits.get(index) ?? 0} unit(s) here`
                : pipe.colors.join(', ') || 'empty'
            }
          />
        )
      })}
    </div>
  )
}
