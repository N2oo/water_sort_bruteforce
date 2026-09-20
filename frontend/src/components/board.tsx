/**
 * The board: the one thing on screen shadcn/ui has no component for.
 *
 * A pipe is a stack of four slots drawn with Tailwind utilities; everything
 * around it is shadcn — the clickable pipe *is* a `Button`, so focus, disabled
 * and keyboard behaviour come from the same place as every other control, and
 * the colors come from the theme variables in `src/index.css`.
 */

import { cn } from 'cn'

import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { colorCode, colorStyle } from '@/game/colors'
import { PIPE_CAPACITY, isCompleted, pour } from '@/game/rules'

export interface BoardPipe {
  id?: string
  label: string
  colors: string[]
}

export type PipeRole = 'idle' | 'selected' | 'target' | 'source' | 'muted'

const ROLE_RING: Record<PipeRole, string> = {
  idle: 'ring-border',
  selected: 'ring-primary ring-2',
  target: 'ring-success ring-2',
  source: 'ring-attention ring-2',
  muted: 'ring-border opacity-50',
}

interface PipeProps {
  label: string
  colors: string[]
  role?: PipeRole
  /** Shade the top units, the ones a pour is about to move. */
  moving?: number
  size?: 'sm' | 'md'
  onClick?: () => void
  disabled?: boolean
  /** What the tooltip says; the colors, bottom first, when left out. */
  hint?: string
}

export function Pipe({
  label,
  colors,
  role = 'idle',
  moving = 0,
  size = 'md',
  onClick,
  disabled = false,
  hint,
}: PipeProps) {
  const done = colors.length === PIPE_CAPACITY && isCompleted(colors)
  const slots = Array.from({ length: PIPE_CAPACITY }, (_, index) => colors[index])
  const unit = size === 'sm' ? 'h-5 w-8' : 'h-8 w-12'

  const tube = (
    <span
      className={cn(
        'flex flex-col-reverse overflow-hidden rounded-t-xs rounded-b-xl bg-muted/40 ring-1 ring-inset transition-shadow',
        ROLE_RING[role],
      )}
    >
      {slots.map((color, index) => {
        const style = color ? colorStyle(color) : undefined
        const isMoving = moving > 0 && color !== undefined && index >= colors.length - moving

        return (
          <span
            key={index}
            className={cn(
              unit,
              'flex items-center justify-center font-mono text-[9px] font-bold tracking-wider',
              !color && 'bg-[repeating-linear-gradient(45deg,var(--muted),var(--muted)_5px,transparent_5px,transparent_10px)]',
              isMoving && 'inset-ring-2 inset-ring-background/80',
            )}
            style={style ? { background: style.fill, color: style.ink } : undefined}
          >
            {color && size === 'md' ? <span className="opacity-70">{colorCode(color)}</span> : null}
          </span>
        )
      })}
    </span>
  )

  const caption = (
    <span className={cn('text-xs', done ? 'text-success font-medium' : 'text-muted-foreground')}>
      {label}
    </span>
  )

  const body = onClick ? (
    <Button
      type="button"
      variant="ghost"
      onClick={onClick}
      disabled={disabled}
      className="h-auto flex-col gap-1.5 p-1 disabled:opacity-100"
    >
      {tube}
      {caption}
    </Button>
  ) : (
    <span className="flex flex-col items-center gap-1.5 p-1">
      {tube}
      {caption}
    </span>
  )

  return (
    <Tooltip>
      <TooltipTrigger asChild>{body}</TooltipTrigger>
      <TooltipContent>{hint ?? (colors.join(' · ') || 'empty')}</TooltipContent>
    </Tooltip>
  )
}

interface BoardProps {
  pipes: BoardPipe[]
  /** Index of the pipe a pour would start from. */
  selected?: number | null
  /** Indices the selected pipe may pour into. */
  targets?: number[]
  /** Drawn as the move about to be played, e.g. by the solution stepper. */
  highlight?: { from: number; to: number } | null
  onPipeClick?: (index: number, pipe: BoardPipe) => void
  disabled?: boolean
  size?: 'sm' | 'md'
}

export function Board({
  pipes,
  selected = null,
  targets = [],
  highlight = null,
  onPipeClick,
  disabled = false,
  size = 'md',
}: BoardProps) {
  const movingUnits =
    highlight && pipes[highlight.from] && pipes[highlight.to]
      ? pour(pipes[highlight.from].colors, pipes[highlight.to].colors).poured
      : 0

  return (
    <div className="flex flex-wrap items-end gap-2">
      {pipes.map((pipe, index) => {
        let role: PipeRole = 'idle'
        if (highlight?.from === index) role = 'source'
        else if (highlight?.to === index) role = 'target'
        else if (selected === index) role = 'selected'
        else if (targets.includes(index)) role = 'target'
        else if (selected !== null) role = 'muted'

        const poured = selected !== null && targets.includes(index)
          ? pour(pipes[selected].colors, pipe.colors).poured
          : 0

        return (
          <Pipe
            key={pipe.id ?? `${pipe.label}-${index}`}
            label={pipe.label}
            colors={pipe.colors}
            role={role}
            moving={highlight?.from === index ? movingUnits : 0}
            size={size}
            disabled={disabled}
            onClick={onPipeClick ? () => onPipeClick(index, pipe) : undefined}
            hint={poured ? `pour ${poured} unit${poured > 1 ? 's' : ''} here` : undefined}
          />
        )
      })}
    </div>
  )
}
