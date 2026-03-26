import { Badge } from '@ui/components/ui/badge'
import { formatExactDateTime, formatRelativeTime, type StoredDeviceHistory } from '@ui/lib/remoteShare'
import { ChevronRight } from 'lucide-react'

export function HistoryRow({
  history,
}: {
  history: StoredDeviceHistory
}) {
  const relativeTime = formatRelativeTime(history.lastUsedAt)
  const exactTime = formatExactDateTime(history.lastUsedAt)

  return (
    <a
      href={`/${history.deviceId}`}
      className="group grid gap-4 border-t border-border/60 py-6 first:border-t-0 first:pt-0 transition-transform duration-300 ease-out hover:translate-x-1"
    >
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 items-center gap-3">
          <Badge variant="secondary" className="rounded-full px-3 py-1 text-[0.68rem] tracking-[0.18em] uppercase">
            Saved
          </Badge>
          <span className="text-xs text-muted-foreground">{relativeTime}</span>
        </div>
        <ChevronRight className="shrink-0 text-muted-foreground transition-transform duration-300 group-hover:translate-x-1" />
      </div>
      <div>
        <p className="break-all text-lg font-semibold tracking-tight text-foreground" title={history.deviceId}>
          {history.deviceId}
        </p>
        <p className="mt-2 text-sm leading-6 text-muted-foreground" title={exactTime}>
          {exactTime}
        </p>
      </div>
    </a>
  )
}
