/**
 * A JSON payload, ready to be curl'd at the API.
 *
 * A `Card` holding a `ScrollArea` and three `Button`s — no styling of its own,
 * it only spares the five pages that show a payload from repeating the same
 * dozen lines.
 */

import { useState } from 'react'
import { Check, ClipboardCopy, Download, Terminal } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { ButtonGroup } from '@/components/ui/button-group'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'

interface JsonCardProps {
  title: string
  /** What the payload is for, usually the endpoint that accepts it. */
  description?: string
  value: unknown
  fileName?: string
  /** A ready to paste `curl` invocation for this payload. */
  curl?: string
}

export function JsonCard({
  title,
  description,
  value,
  fileName = 'scenario.json',
  curl,
}: JsonCardProps) {
  const [copied, setCopied] = useState<'json' | 'curl' | null>(null)
  const text = JSON.stringify(value, null, 2)

  const copy = async (what: 'json' | 'curl', payload: string) => {
    try {
      await navigator.clipboard.writeText(payload)
      setCopied(what)
      window.setTimeout(() => setCopied(null), 1500)
    } catch {
      // Clipboard access can be refused; the text is on screen either way.
    }
  }

  const download = () => {
    const blob = new Blob([text], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = fileName
    link.click()
    URL.revokeObjectURL(url)
  }

  return (
    <Card className="min-w-0">
      <CardHeader>
        <CardTitle className="truncate text-base">{title}</CardTitle>
        {description ? (
          <CardDescription className="truncate font-mono text-xs">{description}</CardDescription>
        ) : null}
      </CardHeader>

      <CardContent>
        <ScrollArea className="bg-code h-64 w-full rounded-md border">
          <pre className="text-code-foreground p-3 font-mono text-xs">{text}</pre>
        </ScrollArea>
      </CardContent>

      <CardFooter>
        <ButtonGroup>
          <Button variant="outline" size="sm" onClick={() => void copy('json', text)}>
            {copied === 'json' ? <Check /> : <ClipboardCopy />}
            JSON
          </Button>
          <Button variant="outline" size="sm" onClick={download}>
            <Download />
            Download
          </Button>
          {curl ? (
            <Button variant="outline" size="sm" onClick={() => void copy('curl', curl)}>
              {copied === 'curl' ? <Check /> : <Terminal />}
              curl
            </Button>
          ) : null}
        </ButtonGroup>
      </CardFooter>
    </Card>
  )
}
