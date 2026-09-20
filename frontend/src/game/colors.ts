/**
 * The twelve colors of `crates/core/src/color.rs`.
 *
 * `COLOR_NAMES` is the canonical spelling the API answers with; `normalizeColor`
 * accepts everything `parse_color` accepts, so a pasted level file using
 * `"gray"` or `"light_blue"` lands on the right swatch. What each color *looks*
 * like belongs to the theme: the values live in `src/index.css`, and this module
 * only points at them.
 */

export const COLOR_NAMES = [
  'Grey',
  'Blue',
  'Lemon',
  'Brown',
  'Green',
  'Red',
  'LightGreen',
  'LightBlue',
  'Pink',
  'Orange',
  'Purple',
  'Yellow',
] as const

export type ColorName = (typeof COLOR_NAMES)[number]

/** The theme variable of each color, and the ink that stays readable on it. */
export const COLOR_STYLES: Record<ColorName, { fill: string; ink: string }> = {
  Grey: { fill: 'var(--pipe-grey)', ink: 'var(--pipe-ink-dark)' },
  Blue: { fill: 'var(--pipe-blue)', ink: 'var(--pipe-ink-light)' },
  Lemon: { fill: 'var(--pipe-lemon)', ink: 'var(--pipe-ink-dark)' },
  Brown: { fill: 'var(--pipe-brown)', ink: 'var(--pipe-ink-light)' },
  Green: { fill: 'var(--pipe-green)', ink: 'var(--pipe-ink-light)' },
  Red: { fill: 'var(--pipe-red)', ink: 'var(--pipe-ink-light)' },
  LightGreen: { fill: 'var(--pipe-light-green)', ink: 'var(--pipe-ink-dark)' },
  LightBlue: { fill: 'var(--pipe-light-blue)', ink: 'var(--pipe-ink-dark)' },
  Pink: { fill: 'var(--pipe-pink)', ink: 'var(--pipe-ink-dark)' },
  Orange: { fill: 'var(--pipe-orange)', ink: 'var(--pipe-ink-dark)' },
  Purple: { fill: 'var(--pipe-purple)', ink: 'var(--pipe-ink-light)' },
  Yellow: { fill: 'var(--pipe-yellow)', ink: 'var(--pipe-ink-dark)' },
}

/** Three letters that fit inside a unit, for reading a board without color. */
export const COLOR_CODES: Record<ColorName, string> = {
  Grey: 'GRY',
  Blue: 'BLU',
  Lemon: 'LMN',
  Brown: 'BRN',
  Green: 'GRN',
  Red: 'RED',
  LightGreen: 'LGN',
  LightBlue: 'LBL',
  Pink: 'PNK',
  Orange: 'ORG',
  Purple: 'PRP',
  Yellow: 'YLW',
}

const ALIASES: Record<string, ColorName> = {
  gray: 'Grey',
  grey: 'Grey',
  light_green: 'LightGreen',
  lightgreen: 'LightGreen',
  light_blue: 'LightBlue',
  lightblue: 'LightBlue',
}

/** The canonical name of a color, or `null` when the palette has no such color. */
export function normalizeColor(raw: string): ColorName | null {
  const key = raw.trim().toLowerCase()
  if (key in ALIASES) return ALIASES[key]
  return COLOR_NAMES.find((name) => name.toLowerCase() === key) ?? null
}

/** An unknown color is drawn as a hatch, so a bad paste is visible at a glance. */
const UNKNOWN = {
  fill: 'repeating-linear-gradient(45deg, var(--muted), var(--muted) 6px, var(--background) 6px, var(--background) 12px)',
  ink: 'var(--foreground)',
}

export function colorStyle(raw: string): { fill: string; ink: string } {
  const name = normalizeColor(raw)
  return name ? COLOR_STYLES[name] : UNKNOWN
}

export function colorCode(raw: string): string {
  const name = normalizeColor(raw)
  return name ? COLOR_CODES[name] : '?'
}

export function isKnownColor(raw: string): boolean {
  return normalizeColor(raw) !== null
}
