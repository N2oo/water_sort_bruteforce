/** The game sessions: resume one where it stands, or drop it. */

import { Link, useNavigate } from 'react-router-dom'

import { useDeleteGame, useGames } from '../api/queries'
import { BoardView } from '../components/BoardView'
import { EmptyState, ErrorNotice, Spinner } from '../components/Notices'

export function GamesPage() {
  const navigate = useNavigate()
  const games = useGames({ limit: 100 })
  const remove = useDeleteGame()

  if (games.isPending) return <Spinner label="loading the games…" />

  return (
    <div className="page">
      <section className="panel">
        <header className="panel__header">
          <div>
            <h2>Games</h2>
            <p className="panel__hint">
              Every session the API is holding, with the board where it was left.
            </p>
          </div>
          <div className="panel__actions">
            <button type="button" className="button--primary" onClick={() => navigate('/design')}>
              start a new one
            </button>
          </div>
        </header>

        <ErrorNotice error={games.error} />
        <ErrorNotice error={remove.error} onDismiss={() => remove.reset()} />

        {games.data && games.data.length === 0 ? (
          <EmptyState title="No game yet">
            <p>
              Design a board in the <Link to="/design">designer</Link> and hit “play this scenario”.
            </p>
          </EmptyState>
        ) : null}

        <div className="cards">
          {games.data?.map((game) => (
            <article key={game.id} className="card">
              <header>
                <h3>{game.name || 'Untitled game'}</h3>
                <p className="panel__hint">
                  {game.status} · {game.moves_played} moves ·{' '}
                  {new Date(game.updated_at).toLocaleString()}
                </p>
              </header>

              <BoardView pipes={game.pipes} compact />

              <footer className="card__actions">
                <button
                  type="button"
                  className="button--primary"
                  onClick={() => navigate(`/games/${game.id}`)}
                >
                  {game.solved ? 'review' : '▶ resume'}
                </button>
                <button
                  type="button"
                  className="button--danger"
                  disabled={remove.isPending}
                  onClick={() => remove.mutate(game.id)}
                >
                  delete
                </button>
              </footer>
            </article>
          ))}
        </div>
      </section>
    </div>
  )
}
