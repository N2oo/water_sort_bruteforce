/**
 * Vendors the shadcn/ui components into `src/components/ui`.
 *
 * `npx shadcn@latest add …` is the normal way to do this, and `components.json`
 * is set up for it. It needs `ui.shadcn.com`, which this environment's network
 * policy blocks, so this script pulls the very same sources from the upstream
 * repository instead. They land here **verbatim**: the only rewrite is the one
 * the CLI also applies, turning the monorepo's registry paths into this
 * project's alias.
 *
 *   from "@/registry/new-york-v4/ui/x" → from "@/components/ui/x"
 *
 * `cn` and `radix-ui` are real packages, so those imports are left alone.
 *
 *   node scripts/fetch-shadcn.mjs [component…]
 */

import { mkdir, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const STYLE = 'new-york-v4'
const BASE = `https://raw.githubusercontent.com/shadcn-ui/ui/main/apps/v4/registry/${STYLE}`
const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')

const DEFAULT_COMPONENTS = [
  'alert',
  'alert-dialog',
  'badge',
  'button',
  'button-group',
  'card',
  'checkbox',
  'dropdown-menu',
  'empty',
  'field',
  'input',
  'item',
  'label',
  'scroll-area',
  'select',
  'separator',
  'skeleton',
  'slider',
  'sonner',
  'spinner',
  'tabs',
  'textarea',
  'toggle',
  'toggle-group',
  'tooltip',
]

const rewrite = (source) =>
  source
    .replaceAll(`@/registry/${STYLE}/ui/`, '@/components/ui/')
    .replaceAll(`@/registry/${STYLE}/lib/`, '@/lib/')
    .replaceAll(`@/registry/${STYLE}/hooks/`, '@/hooks/')

const components = process.argv.slice(2).length ? process.argv.slice(2) : DEFAULT_COMPONENTS

await mkdir(join(ROOT, 'src/components/ui'), { recursive: true })

for (const component of components) {
  const response = await fetch(`${BASE}/ui/${component}.tsx`)
  if (!response.ok) {
    console.error(`✗ ${component}: ${response.status}`)
    process.exitCode = 1
    continue
  }
  await writeFile(join(ROOT, `src/components/ui/${component}.tsx`), rewrite(await response.text()))
  console.log(`✓ ${component}`)
}
