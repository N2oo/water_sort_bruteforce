/**
 * The shell: the navigation, the API health badge, the theme switch, the routes.
 *
 * The user path the app is built around is a straight line — design or paste a
 * board, then either play it or ask for its solution — so the designer is the
 * home page and every other page can be reached from it.
 */

import { Moon, Sun, Beaker, FlaskConical, Library, Zap } from 'lucide-react'
import { useTheme } from 'next-themes'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'

import { API_BASE_URL } from '@/api/client'
import { useHealth } from '@/api/queries'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from 'cn'
import { DesignerPage } from '@/pages/DesignerPage'
import { GamesPage } from '@/pages/GamesPage'
import { PlayPage } from '@/pages/PlayPage'
import { ScenariosPage } from '@/pages/ScenariosPage'
import { SolvePage } from '@/pages/SolvePage'

const NAV = [
  { to: '/design', label: 'Designer', icon: Beaker },
  { to: '/solve', label: 'Solver', icon: Zap },
  { to: '/scenarios', label: 'Scenarios', icon: Library },
  { to: '/games', label: 'Games', icon: FlaskConical },
]

function HealthBadge() {
  const { data, isError, isPending } = useHealth()
  const state = isPending ? 'pending' : isError ? 'down' : data?.status === 'ok' ? 'up' : 'down'

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge variant={state === 'down' ? 'destructive' : 'secondary'}>
          <span
            className={cn(
              'size-2 rounded-full',
              state === 'up' && 'bg-success',
              state === 'pending' && 'bg-muted-foreground animate-pulse',
              state === 'down' && 'bg-current',
            )}
          />
          {state === 'up' ? 'API up' : state === 'pending' ? 'checking…' : 'API unreachable'}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>water-sort-api at {API_BASE_URL || window.location.origin}</TooltipContent>
    </Tooltip>
  )
}

function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme()

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
          aria-label="switch theme"
        >
          <Sun className="hidden dark:block" />
          <Moon className="dark:hidden" />
        </Button>
      </TooltipTrigger>
      <TooltipContent>Switch to {resolvedTheme === 'dark' ? 'light' : 'dark'}</TooltipContent>
    </Tooltip>
  )
}

export default function App() {
  return (
    <div className="flex min-h-svh flex-col">
      <header className="bg-card sticky top-0 z-10 border-b">
        <div className="mx-auto flex w-full max-w-7xl flex-wrap items-center gap-x-6 gap-y-3 px-4 py-3 sm:px-6">
          <div className="flex items-center gap-3">
            <span
              aria-hidden
              className="h-9 w-7 rounded-t-xs rounded-b-xl border-2 bg-[linear-gradient(to_top,var(--pipe-blue)_0_33%,var(--pipe-red)_33%_66%,var(--pipe-yellow)_66%_100%)]"
            />
            <div>
              <h1 className="text-lg leading-tight font-semibold">Water sort</h1>
              <p className="text-muted-foreground text-xs">design a scenario · play it · solve it</p>
            </div>
          </div>

          <nav className="order-last flex w-full items-center gap-0.5 sm:order-none sm:ml-auto sm:w-auto sm:gap-1">
            {NAV.map(({ to, label, icon: Icon }) => (
              <NavLink key={to} to={to} className="flex-1 sm:flex-none">
                {({ isActive }) => (
                  <Button
                    variant={isActive ? 'secondary' : 'ghost'}
                    size="sm"
                    className="w-full justify-center px-2 text-xs sm:px-3 sm:text-sm"
                    asChild
                  >
                    <span>
                      <Icon />
                      {label}
                    </span>
                  </Button>
                )}
              </NavLink>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-2 sm:ml-0">
            <HealthBadge />
            <Separator orientation="vertical" className="h-6" />
            <ThemeToggle />
          </div>
        </div>
      </header>

      <main className="mx-auto w-full max-w-7xl flex-1 px-4 py-6 sm:px-6">
        <Routes>
          <Route path="/" element={<Navigate to="/design" replace />} />
          <Route path="/design" element={<DesignerPage />} />
          <Route path="/solve" element={<SolvePage />} />
          <Route path="/scenarios" element={<ScenariosPage />} />
          <Route path="/games" element={<GamesPage />} />
          <Route path="/games/:gameId" element={<PlayPage />} />
          <Route path="*" element={<Navigate to="/design" replace />} />
        </Routes>
      </main>

      <footer className="text-muted-foreground border-t px-4 py-4 text-center text-xs sm:px-6">
        Backed by <code className="font-mono">water-sort-api</code> — the rules and the solver live
        in <code className="font-mono">crates/core</code>.
      </footer>
    </div>
  )
}
