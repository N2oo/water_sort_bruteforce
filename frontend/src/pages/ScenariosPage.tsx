/** The saved boards: open one in the designer, play it, solve it, drop it. */

import { Copy, Library, Play, Plus, Trash2, Zap } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { useCreateGame, useDeleteScenario, useScenarios } from '@/api/queries'
import { Board } from '@/components/board'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { Skeleton } from '@/components/ui/skeleton'
import { errorCode, explain } from '@/lib/errors'
import { draftFromPipes } from '@/game/scenario'
import { useDraft } from '@/state/draft'

export function ScenariosPage() {
  const navigate = useNavigate()
  const { setDraft } = useDraft()

  const scenarios = useScenarios({ limit: 100 })
  const createGame = useCreateGame()
  const remove = useDeleteScenario()

  const fail = (error: unknown) => toast.error(errorCode(error), { description: explain(error) })

  return (
    <div className="flex flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Scenarios</CardTitle>
          <CardDescription>
            Boards saved through <code className="font-mono">POST /api/v1/scenarios</code>. They
            outlive a game.
          </CardDescription>
          <CardAction>
            <Button onClick={() => navigate('/design')}>
              <Plus />
              Design a new one
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent>
          {scenarios.isPending ? (
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
              {[0, 1, 2].map((key) => (
                <Skeleton key={key} className="h-64 w-full" />
              ))}
            </div>
          ) : null}

          {scenarios.data && scenarios.data.length === 0 ? (
            <Empty className="border border-dashed">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <Library />
                </EmptyMedia>
                <EmptyTitle>Nothing saved yet</EmptyTitle>
                <EmptyDescription>Design a board and hit “Save as scenario”.</EmptyDescription>
              </EmptyHeader>
              <Button variant="outline" onClick={() => navigate('/design')}>
                Open the designer
              </Button>
            </Empty>
          ) : null}

          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {scenarios.data?.map((scenario) => (
              <Card key={scenario.id} className="bg-surface">
                <CardHeader>
                  <CardTitle className="text-base">{scenario.name}</CardTitle>
                  <CardDescription>
                    {scenario.pipes.length} pipes · {new Date(scenario.created_at).toLocaleString()}
                    {scenario.description ? ` · ${scenario.description}` : ''}
                  </CardDescription>
                </CardHeader>

                <CardContent>
                  <Board pipes={scenario.pipes} size="sm" />
                </CardContent>

                <CardFooter className="mt-auto flex-wrap gap-2">
                  <Button
                    size="sm"
                    disabled={createGame.isPending}
                    onClick={() =>
                      createGame.mutate(
                        { scenario_id: scenario.id, name: scenario.name },
                        { onSuccess: (game) => navigate(`/games/${game.id}`), onError: fail },
                      )
                    }
                  >
                    <Play />
                    Play
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => navigate(`/solve?scenario=${scenario.id}`)}
                  >
                    <Zap />
                    Solve
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      setDraft(
                        draftFromPipes(scenario.pipes, scenario.name, scenario.description ?? ''),
                      )
                      navigate('/design')
                    }}
                  >
                    <Copy />
                    Edit a copy
                  </Button>

                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button size="icon-sm" variant="ghost" aria-label={`delete ${scenario.name}`}>
                        <Trash2 className="text-destructive" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Delete “{scenario.name}”?</AlertDialogTitle>
                        <AlertDialogDescription>
                          The scenario is dropped from the API for good. Games already started from
                          it keep their own copy of the board.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                          onClick={() =>
                            remove.mutate(scenario.id, {
                              onSuccess: () => toast.success(`“${scenario.name}” deleted`),
                              onError: fail,
                            })
                          }
                        >
                          Delete
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </CardFooter>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
