/**
 * The levels shipped in `levels/`, inlined so the designer has something to
 * open on a first run. They are the same boards the CLI has always solved.
 */

import type { Draft } from './scenario'

export const SAMPLES: Draft[] = [
  {
    name: "Level 145",
    description: 'Shipped with the repository, in levels/level145.json',
    pipes: [
      { label: "P1", colors: ["Brown", "Lemon", "Blue", "Grey"] },
      { label: "P2", colors: ["Green", "Red", "Red", "Green"] },
      { label: "P3", colors: ["Pink", "Blue", "LightGreen", "LightBlue"] },
      { label: "P4", colors: ["LightGreen", "Grey", "Brown", "Orange"] },
      { label: "P5", colors: ["Blue", "Orange", "LightBlue", "Purple"] },
      { label: "P6", colors: ["Yellow", "Yellow", "Lemon", "LightBlue"] },
      { label: "P7", colors: ["LightGreen", "Blue", "Purple", "Grey"] },
      { label: "P8", colors: ["Pink", "Red", "Green", "Orange"] },
      { label: "P9", colors: ["LightBlue", "Yellow", "Grey", "Brown"] },
      { label: "P10", colors: ["Purple", "Pink", "Brown", "LightGreen"] },
      { label: "P11", colors: ["Red", "Lemon", "Green", "Purple"] },
      { label: "P12", colors: ["Yellow", "Orange", "Lemon", "Pink"] },
      { label: "P13", colors: [] },
      { label: "P14", colors: [] },
    ],
  },
  {
    name: "Level 146",
    description: 'Shipped with the repository, in levels/level146.json',
    pipes: [
      { label: "P1", colors: ["Red", "Orange", "Pink", "Orange"] },
      { label: "P2", colors: ["Gray", "Lemon", "Pink", "Red"] },
      { label: "P3", colors: ["LightGreen", "Purple", "LightBlue", "Lemon"] },
      { label: "P4", colors: ["Purple", "Pink", "Blue", "Orange"] },
      { label: "P5", colors: ["LightBlue", "Purple", "LightGreen", "Gray"] },
      { label: "P6", colors: ["LightGreen", "Red", "Purple", "Blue"] },
      { label: "P7", colors: ["LightGreen", "Orange", "LightBlue", "Gray"] },
      { label: "P8", colors: ["Blue", "Gray", "Red", "LightBlue"] },
      { label: "P9", colors: ["Lemon", "Blue", "Pink", "Lemon"] },
      { label: "P10", colors: [] },
      { label: "P11", colors: [] },
    ],
  },
]
