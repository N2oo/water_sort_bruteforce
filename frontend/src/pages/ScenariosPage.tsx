/** The saved boards: open one in the designer, play it, solve it, drop it. */

import { useNavigate } from 'react-router-dom'

import { useCreateGame, useDeleteScenario, useScenarios } from '../api/queries'
import { BoardView } from '../components/BoardView'
import { EmptyState, ErrorNotice, Spinner } from '../components/Notices'
import { draftFromPipes } from '../game/scenario'
import { useDraft } from '../state/draft'

export function ScenariosPage() {
  const navigate = useNavigate()
  const { setDraft } = useDraft()

  const scenarios = useScenarios({ limit: 100 })
  const createGame = useCreateGame()
  const remove = useDeleteScenario()

  if (scenarios.isPending) return <Spinner label="loading the scenarios…" />

  return (
    <div className="page">
      <section className="panel">
        <header className="panel__header">
          <div>
            <h2>Scenarios</h2>
            <p className="panel__hint">
              Boards saved through <code>POST /api/v1/scenarios</code>. They outlive a game.
            </p>
          </div>
          <div className="panel__actions">
            <button type="button" className="button--primary" onClick={() => navigate('/design')}>
              design a new one
            </button>
          </div>
        </header>

        <ErrorNotice error={scenarios.error} />
        <ErrorNotice error={createGame.error} onDismiss={() => createGame.reset()} />
        <ErrorNotice error={remove.error} onDismiss={() => remove.reset()} />

        {scenarios.data && scenarios.data.length === 0 ? (
          <EmptyState title="Nothing saved yet">
            <p>Design a board and hit “save as scenario”.</p>
          </EmptyState>
        ) : null}

        <div className="cards">
          {scenarios.data?.map((scenario) => (
            <article key={scenario.id} className="card">
              <header>
                <h3>{scenario.name}</h3>
                <p className="panel__hint">
                  {scenario.pipes.length} pipes · {new Date(scenario.created_at).toLocaleString()}
                </p>
                {scenario.description ? <p>{scenario.description}</p> : null}
              </header>

              <BoardView pipes={scenario.pipes} compact />

              <footer className="card__actions">
                <button
                  type="button"
                  className="button--primary"
                  disabled={createGame.isPending}
                  onClick={() =>
                    createGame.mutate(
                      { scenario_id: scenario.id, name: scenario.name },
                      { onSuccess: (game) => navigate(`/games/${game.id}`) },
                    )
                  }
                >
                  ▶ play
                </button>
                <button type="button" onClick={() => navigate(`/solve?scenario=${scenario.id}`)}>
                  ⚡ solve
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setDraft(draftFromPipes(scenario.pipes, scenario.name, scenario.description ?? ''))
                    navigate('/design')
                  }}
                >
                  edit a copy
                </button>
                <button
                  type="button"
                  className="button--danger"
                  disabled={remove.isPending}
                  onClick={() => remove.mutate(scenario.id)}
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
