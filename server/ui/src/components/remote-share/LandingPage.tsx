import type { StoredDeviceHistory } from '@ui/lib/remoteShare'

import { Alert, AlertDescription, AlertTitle } from '@ui/components/ui/alert'
import { Badge } from '@ui/components/ui/badge'
import { Skeleton } from '@ui/components/ui/skeleton'
import { formatCount, formatRelativeTime } from '@ui/lib/remoteShare'
import { RefreshCw, TriangleAlert } from 'lucide-react'

import { HistoryRow } from './HistoryRow'
import { HeroMetric, SectionLabel } from './Shared'

export function LandingPage({
  error,
  heroDescription,
  heroTitle,
  historyDevices,
  isLoadingHistory,
  latestHistoryAt,
  primaryActionLabel,
}: {
  error: string
  heroDescription: string
  heroTitle: string
  historyDevices: StoredDeviceHistory[]
  isLoadingHistory: boolean
  latestHistoryAt: string | null
  primaryActionLabel: string
}) {
  return (
    <main className="min-h-screen bg-background text-foreground">
      <section className="remote-hero">
        <div className="remote-hero__backdrop" />
        <div className="remote-hero__glow remote-hero__glow--left" />
        <div className="remote-hero__glow remote-hero__glow--right" />

        <div className="remote-shell">
          <div className="remote-hero__content">
            <div className="remote-hero__brand animate-in fade-in slide-in-from-bottom-4 duration-700">
              <img
                alt="Launcherg"
                className="remote-brand-mark"
                src="/icon.png"
              />
              <div className="min-w-0">
                <p className="remote-brand-name">Launcherg</p>
                <p className="remote-brand-subtitle">Remote Share</p>
              </div>
            </div>

            <div className="remote-hero__copy animate-in fade-in slide-in-from-bottom-4 duration-700 delay-100">
              <p className="remote-kicker">Remote access for your registered library</p>
              <h1 className="remote-display">{heroTitle}</h1>
              <p className="remote-lead">{heroDescription}</p>

              <div className="flex flex-wrap items-center gap-3">
                <a
                  href="#connection-surface"
                  className="inline-flex h-12 items-center justify-center rounded-full bg-white px-6 text-sm font-medium text-[#102137] transition-colors hover:bg-white/92"
                >
                  {primaryActionLabel}
                </a>

                <a
                  href="#connection-surface"
                  className="inline-flex h-12 items-center justify-center rounded-full border border-white/16 px-5 text-sm font-medium text-white transition-colors hover:bg-white/8"
                >
                  接続面へ移動
                </a>
              </div>

              <div className="remote-hero__meta">
                <HeroMetric label="Saved" value={historyDevices.length > 0 ? `${formatCount(historyDevices.length)} 件` : '0 件'} />
                <HeroMetric
                  label="Last update"
                  value={latestHistoryAt ? formatRelativeTime(latestHistoryAt) : 'まだありません'}
                />
                <HeroMetric
                  label="Mode"
                  value={historyDevices.length > 0 ? 'reconnect ready' : 'first connect'}
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      <section id="connection-surface" className="remote-surface">
        <div className="remote-shell">
          {error && (
            <Alert variant="destructive" className="mb-8 border-destructive/30 bg-destructive/6 text-foreground">
              <TriangleAlert />
              <AlertTitle>接続できませんでした</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <section className="remote-section">
            <div className="remote-section__intro">
              <div>
                <SectionLabel>Saved devices</SectionLabel>
                <h2 className="mt-4 text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
                  保存済みの接続先を選ぶ
                </h2>
              </div>

              <div className="remote-section__summary">
                <p className="text-sm leading-7 text-muted-foreground sm:text-base">
                  最近使った順に並べています。前に使ったPCへすぐ戻りたいときは、上から選ぶだけで十分です。
                </p>
                <div className="flex flex-wrap gap-2">
                  <Badge variant="secondary" className="rounded-full px-3 py-1">
                    {historyDevices.length > 0 ? '保存済みあり' : '未接続'}
                  </Badge>
                </div>
              </div>
            </div>

            <div className="remote-panel">
              {isLoadingHistory && (
                <div className="space-y-4">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <RefreshCw className="animate-spin" />
                    接続先を確認中...
                  </div>
                  <Skeleton className="h-24 w-full rounded-[1.6rem]" />
                  <Skeleton className="h-24 w-full rounded-[1.6rem]" />
                </div>
              )}

              {!isLoadingHistory && historyDevices.length > 0 && (
                <div>
                  {historyDevices.map(history => (
                    <HistoryRow key={history.deviceId} history={history} />
                  ))}
                </div>
              )}

              {!isLoadingHistory && historyDevices.length === 0 && !error && (
                <div className="space-y-4">
                  <p className="text-lg font-semibold tracking-tight text-foreground">
                    保存済みの接続先はまだありません。
                  </p>
                  <p className="max-w-2xl text-sm leading-7 text-muted-foreground sm:text-base">
                    PC版アプリで共有URLを開くか、QRコードを読み取って最初の接続を済ませると、ここに履歴が追加されます。
                  </p>
                </div>
              )}
            </div>
          </section>
        </div>
      </section>
    </main>
  )
}
