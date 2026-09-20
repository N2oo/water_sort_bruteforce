/**
 * The twelve colors of `crates/core/src/color.rs`, with the CSS to draw them.
 *
 * `COLOR_NAMES` is the canonical spelling the API answers with; `normalizeColor`
 * accepts everything `parse_color` accepts, so a pasted level file using
 * `"gray"` or `"light_blue"` lands on the right swatch.
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

/** What each color looks like, and what to write on top of it. */
export const COLOR_STYLES: Record<ColorName, { fill: string; ink: string }> = {
  Grey: { fill: '#8d949e', ink: '#12151a' },
  Blue: { fill: '#2563eb', ink: '#f8fafc' },
  Lemon: { fill: '#d9e021', ink: '#20240a' },
  Brown: { fill: '#8b5a2b', ink: '#fdf6ee' },
  Green: { fill: '#177245', ink: '#f0fff6' },
  Red: { fill: '#d92626', ink: '#fff5f5' },
  LightGreen: { fill: '#7ed957', ink: '#12240a' },
  LightBlue: { fill: '#67c7f0', ink: '#062030' },
  Pink: { fill: '#f57ec1', ink: '#3a0b26' },
  Orange: { fill: '#f28c28', ink: '#2c1403' },
  Purple: { fill: '#7b3fd4', ink: '#f7f2ff' },
  Yellow: { fill: '#f5c518', ink: '#2a1f02' },
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

export function colorStyle(raw: string): { fill: string; ink: string } {
  const name = normalizeColor(raw)
  return name ? COLOR_STYLES[name] : { fill: 'repeating-linear-gradient(45deg, #444, #444 6px, #222 6px, #222 12px)', ink: '#fff' }
}

export function colorCode(raw: string): string {
  const name = normalizeColor(raw)
  return name ? COLOR_CODES[name] : '?'
}

export function isKnownColor(raw: string): boolean {
  return normalizeColor(raw) !== null
}
