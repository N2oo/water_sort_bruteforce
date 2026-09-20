/**
 * The shell: the navigation, the API health badge, and the routes.
 *
 * The user path the app is built around is a straight line — design or paste a
 * board, then either play it or ask for its solution — so the designer is the
 * home page and every other page can be reached from it.
 */

import { NavLink, Navigate, Route, Routes } from 'react-router-dom'

import { useHealth } from './api/queries'
import { API_BASE_URL } from './api/client'
import { DesignerPage } from './pages/DesignerPage'
import { PlayPage } from './pages/PlayPage'
import { SolvePage } from './pages/SolvePage'
import { ScenariosPage } from './pages/ScenariosPage'
import { GamesPage } from './pages/GamesPage'

function HealthBadge() {
  const { data, isError, isPending } = useHealth()
  const state = isPending ? 'pending' : isError ? 'down' : data?.status === 'ok' ? 'up' : 'down'
  const target = API_BASE_URL || window.location.origin

  return (
    <span className={`health health--${state}`} title={`water-sort-api at ${target}`}>
      <span className="health__dot" />
      {state === 'up' ? 'API up' : state === 'pending' ? 'checking…' : 'API unreachable'}
    </span>
  )
}

export default function App() {
  return (
    <div className="app">
      <header className="app__header">
        <div className="app__brand">
          <span className="app__logo" aria-hidden="true" />
          <div>
            <h1>Water sort</h1>
            <p>design a scenario · play it · solve it</p>
          </div>
        </div>

        <nav className="app__nav">
          <NavLink to="/design" end>
            Designer
          </NavLink>
          <NavLink to="/solve">Solver</NavLink>
          <NavLink to="/scenarios">Scenarios</NavLink>
          <NavLink to="/games">Games</NavLink>
        </nav>

        <HealthBadge />
      </header>

      <main className="app__main">
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

      <footer className="app__footer">
        <span>
          Backed by <code>water-sort-api</code> — the rules and the solver live in{' '}
          <code>crates/core</code>.
        </span>
      </footer>
    </div>
  )
}
