import type { DeviceWorksListItem } from '@server/shared/schema'

import { Button } from '@ui/components/ui/button'
import { getWorkDetailPath } from '@ui/lib/remoteShare'
import { cn } from '@ui/lib/utils'
import { ImageIcon, LayoutGrid, Rows3 } from 'lucide-react'

import type { ViewMode } from '@ui/lib/remoteShare'

function getAspectRatio(work: DeviceWorksListItem): string {
  if (work.width && work.height) {
    return `${work.width} / ${work.height}`
  }

  return '4 / 3'
}

function WorkImage({
  work,
  className,
}: {
  work: DeviceWorksListItem
  className?: string
}) {
  const aspectRatio = getAspectRatio(work)

  return (
    <div className={cn('remote-work-image-frame', className)} style={{ aspectRatio }}>
      {work.imageUrl
        ? (
          <img
            alt={work.title}
            className="remote-work-image"
            src={work.imageUrl}
          />
        )
        : (
          <div className="remote-work-image remote-work-image--placeholder">
            <ImageIcon />
          </div>
        )}
    </div>
  )
}

function WorkMasonryCard({
  deviceId,
  work,
}: {
  deviceId: string
  work: DeviceWorksListItem
}) {
  return (
    <a className="remote-masonry-card group block" href={getWorkDetailPath(deviceId, work.workId)}>
      <article>
        <WorkImage work={work} />
      </article>
    </a>
  )
}

function WorkListRow({
  deviceId,
  work,
}: {
  deviceId: string
  work: DeviceWorksListItem
}) {
  return (
    <a className="remote-list-row" href={getWorkDetailPath(deviceId, work.workId)}>
      <WorkImage work={work} className="remote-list-row__image-frame" />
      <div className="min-w-0">
        <h2 className="truncate text-base font-semibold text-foreground sm:text-lg">{work.title}</h2>
      </div>
    </a>
  )
}

export function ViewModeToggle({
  viewMode,
  onChange,
}: {
  viewMode: ViewMode
  onChange: (viewMode: ViewMode) => void
}) {
  return (
    <div className="inline-flex items-center gap-1 rounded-full border border-border/80 bg-background/90 p-1">
      <Button
        type="button"
        variant={viewMode === 'masonry' ? 'secondary' : 'ghost'}
        size="sm"
        className="rounded-full px-3"
        onClick={() => onChange('masonry')}
      >
        <LayoutGrid />
        Masonry
      </Button>
      <Button
        type="button"
        variant={viewMode === 'list' ? 'secondary' : 'ghost'}
        size="sm"
        className="rounded-full px-3"
        onClick={() => onChange('list')}
      >
        <Rows3 />
        List
      </Button>
    </div>
  )
}

export function WorksPresentation({
  deviceId,
  viewMode,
  works,
}: {
  deviceId: string
  viewMode: ViewMode
  works: DeviceWorksListItem[]
}) {
  if (viewMode === 'list') {
    return (
      <div className="space-y-3">
        {works.map(work => (
          <WorkListRow key={work.workId} deviceId={deviceId} work={work} />
        ))}
      </div>
    )
  }

  return (
    <div className="remote-masonry">
      {works.map(work => (
        <WorkMasonryCard key={work.workId} deviceId={deviceId} work={work} />
      ))}
    </div>
  )
}
