import type { DeviceWorksListItem, LaunchWorkResponse } from '@server/shared/schema'

import { api } from '@ui/api'
import { Alert, AlertDescription, AlertTitle } from '@ui/components/ui/alert'
import { Button } from '@ui/components/ui/button'
import { formatExactDateTime, getLibraryPath } from '@ui/lib/remoteShare'
import { cn } from '@ui/lib/utils'
import { ArrowLeft, ExternalLink, ImageIcon, Rocket, TriangleAlert } from 'lucide-react'
import { useState } from 'react'

function WorkHeroImage({
  work,
}: {
  work: DeviceWorksListItem
}) {
  return (
    <div className="remote-work-detail__visual" style={{ aspectRatio: work.width && work.height ? `${work.width} / ${work.height}` : '4 / 3' }}>
      {work.imageUrl
        ? (
          <img alt={work.title} className="remote-work-detail__image" src={work.imageUrl} />
        )
        : (
          <div className="remote-work-detail__image remote-work-detail__image--placeholder">
            <ImageIcon />
          </div>
        )}
    </div>
  )
}

function WorkDetailLink({
  href,
  label,
}: {
  href: string | null | undefined
  label: string
}) {
  if (!href) {
    return null
  }

  return (
    <a className="remote-work-detail__link" href={href} rel="noreferrer" target="_blank">
      <span>{label}</span>
      <ExternalLink className="size-4" />
    </a>
  )
}

export function LaunchResultAlert({
  result,
}: {
  result: LaunchWorkResponse | null
}) {
  if (!result) {
    return null
  }

  if (result.ok) {
    return (
      <Alert className="border-primary/20 bg-primary/6">
        <AlertTitle>起動しました</AlertTitle>
        <AlertDescription>{result.message}</AlertDescription>
      </Alert>
    )
  }

  return (
    <Alert variant="destructive" className="border-destructive/30 bg-destructive/6 text-foreground">
      <TriangleAlert />
      <AlertTitle>起動できませんでした</AlertTitle>
      <AlertDescription>{result.message}</AlertDescription>
    </Alert>
  )
}

export function WorkDetailPanel({
  deviceId,
  lastSyncedAt,
  work,
}: {
  deviceId: string
  lastSyncedAt: string | null
  work: DeviceWorksListItem | null
}) {
  const [launchResult, setLaunchResult] = useState<LaunchWorkResponse | null>(null)
  const [isLaunching, setIsLaunching] = useState(false)

  if (!work) {
    return (
      <section className="remote-gate">
        <p className="text-lg font-semibold text-foreground">作品が見つかりません</p>
        <p className="mt-3 text-sm leading-7 text-muted-foreground">
          URL が古いか、同期済みの一覧にこの作品が含まれていません。
        </p>
        <a
          className="mt-6 inline-flex h-11 items-center justify-center gap-1.5 rounded-full bg-secondary px-5 text-sm font-medium text-secondary-foreground transition-colors hover:bg-secondary/80"
          href={getLibraryPath(deviceId)}
        >
          <ArrowLeft />
          ライブラリに戻る
        </a>
      </section>
    )
  }

  const launch = async () => {
    setIsLaunching(true)
    setLaunchResult(null)

    try {
      const result = await api.launchWork(deviceId, work.workId)
      setLaunchResult(result)
    }
    catch (cause) {
      setLaunchResult({
        ok: false,
        reasonCode: 'unknown',
        message: 'ゲームを起動できませんでした。しばらくしてから再試行してください。',
        detail: cause instanceof Error ? cause.message : String(cause),
      })
    }
    finally {
      setIsLaunching(false)
    }
  }

  return (
    <section className="remote-work-detail">
      <div className="remote-work-detail__header">
        <a
          className="inline-flex h-11 items-center justify-center gap-1.5 rounded-full bg-secondary px-5 text-sm font-medium text-secondary-foreground transition-colors hover:bg-secondary/80"
          href={getLibraryPath(deviceId)}
        >
          <ArrowLeft />
          Library
        </a>
      </div>

      <div className="remote-work-detail__content">
        <WorkHeroImage work={work} />

        <section className="remote-work-detail__body">
          <div>
            <p className="text-[0.72rem] tracking-[0.28em] text-muted-foreground uppercase">Work Detail</p>
            <h1 className="mt-4 text-3xl font-semibold tracking-tight text-foreground sm:text-5xl">{work.title}</h1>
            <p className="mt-4 text-sm leading-7 text-muted-foreground">
              最終同期:
              {' '}
              {formatExactDateTime(lastSyncedAt)}
            </p>
          </div>

          <div className="remote-work-detail__links">
            <WorkDetailLink href={work.officialUrl} label="Official" />
            <WorkDetailLink href={work.erogamescapeUrl} label="ErogameScape" />
            <WorkDetailLink href={work.seiyaUrl} label="誠也の部屋" />
          </div>

          <div className="remote-work-detail__actions">
            <Button
              className="h-12 rounded-full px-6"
              disabled={isLaunching}
              onClick={() => void launch()}
              size="lg"
              type="button"
            >
              <Rocket className={cn(isLaunching && 'animate-pulse')} />
              {isLaunching ? '送信中...' : '起動する'}
            </Button>
          </div>

          <LaunchResultAlert result={launchResult} />
        </section>
      </div>
    </section>
  )
}
