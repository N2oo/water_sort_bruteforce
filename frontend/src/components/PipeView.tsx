/** One pipe, drawn bottom first, the way the API stores it. */

import { colorCode, colorStyle } from '../game/colors'
import { PIPE_CAPACITY, isCompleted } from '../game/rules'

export type PipeRole = 'idle' | 'selected' | 'target' | 'source' | 'muted'

interface PipeViewProps {
  label: string
  colors: string[]
  role?: PipeRole
  /** Highlight the units the next pour would move. */
  moving?: number
  onClick?: () => void
  disabled?: boolean
  title?: string
}

export function PipeView({
  label,
  colors,
  role = 'idle',
  moving = 0,
  onClick,
  disabled = false,
  title,
}: PipeViewProps) {
  const done = colors.length === PIPE_CAPACITY && isCompleted(colors)
  const slots = Array.from({ length: PIPE_CAPACITY }, (_, index) => colors[index])

  const className = [
    'pipe',
    `pipe--${role}`,
    done ? 'pipe--done' : '',
    onClick ? 'pipe--clickable' : '',
  ]
    .filter(Boolean)
    .join(' ')

  const content = (
    <>
      <div className="pipe__tube">
        {/* Bottom first in the data, so the column is reversed for the eye. */}
        {slots
          .map((color, index) => ({ color, index }))
          .reverse()
          .map(({ color, index }) => {
            const style = color ? colorStyle(color) : null
            const isMoving = moving > 0 && color !== undefined && index >= colors.length - moving
            return (
              <div
                key={index}
                className={`pipe__unit ${color ? 'pipe__unit--filled' : 'pipe__unit--empty'} ${
                  isMoving ? 'pipe__unit--moving' : ''
                }`}
                style={style ? { background: style.fill, color: style.ink } : undefined}
                title={color}
              >
                {color ? <span className="pipe__unit-label">{colorCode(color)}</span> : null}
              </div>
            )
          })}
      </div>
      <div className="pipe__label">{label}</div>
    </>
  )

  if (!onClick) {
    return (
      <div className={className} title={title}>
        {content}
      </div>
    )
  }

  return (
    <button type="button" className={className} onClick={onClick} disabled={disabled} title={title}>
      {content}
    </button>
  )
}
