/** The game sessions: resume one where it stands, or drop it. */

import { FlaskConical, Play, Plus, Trash2 } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { useDeleteGame, useGames } from '@/api/queries'
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
import { Badge } from '@/components/ui/badge'
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

export function GamesPage() {
  const navigate = useNavigate()
  const games = useGames({ limit: 100 })
  const remove = useDeleteGame()

  const fail = (error: unknown) => toast.error(errorCode(error), { description: explain(error) })

  return (
    <Card>
      <CardHeader>
        <CardTitle>Games</CardTitle>
        <CardDescription>
          Every session the API is holding, with the board where it was left.
        </CardDescription>
        <CardAction>
          <Button onClick={() => navigate('/design')}>
            <Plus />
            Start a new one
          </Button>
        </CardAction>
      </CardHeader>

      <CardContent>
        {games.isPending ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[0, 1, 2].map((key) => (
              <Skeleton key={key} className="h-56 w-full" />
            ))}
          </div>
        ) : null}

        {games.data && games.data.length === 0 ? (
          <Empty className="border border-dashed">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <FlaskConical />
              </EmptyMedia>
              <EmptyTitle>No game yet</EmptyTitle>
              <EmptyDescription>
                Design a board in the designer and hit “Play this scenario”.
              </EmptyDescription>
            </EmptyHeader>
            <Button variant="outline" onClick={() => navigate('/design')}>
              Open the designer
            </Button>
          </Empty>
        ) : null}

        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          {games.data?.map((game) => (
            <Card key={game.id} className="bg-surface">
              <CardHeader>
                <CardTitle className="text-base">{game.name || 'Untitled game'}</CardTitle>
                <CardDescription>
                  {game.moves_played} moves · {new Date(game.updated_at).toLocaleString()}
                </CardDescription>
                <CardAction>
                  <Badge variant={game.solved ? 'default' : 'secondary'}>{game.status}</Badge>
                </CardAction>
              </CardHeader>

              <CardContent>
                <Board pipes={game.pipes} size="sm" />
              </CardContent>

              <CardFooter className="mt-auto gap-2">
                <Button size="sm" onClick={() => navigate(`/games/${game.id}`)}>
                  <Play />
                  {game.solved ? 'Review' : 'Resume'}
                </Button>

                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button size="icon-sm" variant="ghost" aria-label="delete this game">
                      <Trash2 className="text-destructive" />
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Delete this game?</AlertDialogTitle>
                      <AlertDialogDescription>
                        The session and its move history are dropped from the API for good.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={() =>
                          remove.mutate(game.id, {
                            onSuccess: () => toast.success('Game deleted'),
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
  )
}
