import type { DeviceWorksListOutput } from '@server/shared/schema'
import type { FormEvent } from 'react'

import { Alert, AlertDescription, AlertTitle } from '@ui/components/ui/alert'
import { Badge } from '@ui/components/ui/badge'
import { Button } from '@ui/components/ui/button'
import { Input } from '@ui/components/ui/input'
import { Skeleton } from '@ui/components/ui/skeleton'
import {
  formatCount,
  formatExactDateTime,
  formatRelativeTime,
  getSelectedWorkId,
  type ViewMode,
} from '@ui/lib/remoteShare'
import { RefreshCw, ShieldCheck, TriangleAlert } from 'lucide-react'

import { SectionLabel } from './Shared'
import { WorkDetailPanel } from './WorkDetailPanel'
import { ViewModeToggle, WorksPresentation } from './WorkViews'

export function DeviceWorkspace({
  deviceId,
  deviceSecret,
  error,
  isLoading,
  isRestoring,
  onSecretChange,
  onSubmit,
  setViewMode,
  viewMode,
  visibleWorks,
}: {
  deviceId: string
  deviceSecret: string
  error: string
  isLoading: boolean
  isRestoring: boolean
  onSecretChange: (value: string) => void
  onSubmit: (event: FormEvent<HTMLFormElement>) => void
  setViewMode: (mode: ViewMode) => void
  viewMode: ViewMode
  visibleWorks: DeviceWorksListOutput | null
}) {
  const workCount = visibleWorks ? formatCount(visibleWorks.works.length) : '0'
  const selectedWorkId = getSelectedWorkId()
  const selectedWork = selectedWorkId && visibleWorks
    ? visibleWorks.works.find(work => work.workId === selectedWorkId) ?? null
    : null

  return (
    <main className="remote-library">
      <header className="remote-library__header">
        <div className="remote-shell">
          <div className="remote-library__header-inner">
            <div className="flex items-center gap-3">
              <img alt="Launcherg" className="remote-library__mark" src="/icon.png" />
              <div className="min-w-0">
                <p className="text-xl font-semibold tracking-tight text-foreground sm:text-2xl">Launcherg</p>
                <p className="mt-1 text-[0.68rem] tracking-[0.28em] text-muted-foreground uppercase">
                  Remote Library
                </p>
              </div>
            </div>
          </div>
        </div>
      </header>

      <section className="remote-library__body">
        <div className="remote-shell">
          {error && (
            <Alert variant="destructive" className="mb-8 border-destructive/30 bg-destructive/6 text-foreground">
              <TriangleAlert />
              <AlertTitle>接続できませんでした</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {isRestoring && !error && (
            <section className="remote-gate">
              <div className="space-y-5">
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <RefreshCw className="animate-spin" />
                  保存済みの接続情報を確認中...
                </div>
                <Skeleton className="h-11 w-full rounded-full" />
                <Skeleton className="h-64 w-full rounded-[1.6rem]" />
              </div>
            </section>
          )}

          {!isRestoring && !visibleWorks && (
            <section className="remote-gate">
              <div className="remote-gate__intro">
                <SectionLabel>Connection</SectionLabel>
                <h1 className="mt-4 text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
                  この端末でライブラリを開く
                </h1>
                <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground sm:text-base">
                  共有キーを入力すると、このブラウザに接続先を保存します。接続が完了した後は、このページをそのまま作品ビューとして使えます。
                </p>
              </div>

              <form className="mt-8 grid gap-6 lg:grid-cols-[minmax(0,1fr)_15rem] lg:items-end" onSubmit={onSubmit}>
                <div className="space-y-5">
                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground" htmlFor="device-secret-ready">
                      共有キー
                    </label>
                    <Input
                      id="device-secret-ready"
                      value={deviceSecret}
                      onChange={event => onSecretChange(event.target.value)}
                      placeholder="PC版アプリで設定した共有キー"
                      type="password"
                      autoComplete="off"
                      className="h-12 rounded-full border-border/70 bg-background px-5"
                    />
                  </div>
                  <p className="text-sm leading-7 text-muted-foreground">
                    認証に成功した共有キーだけをこのブラウザに保存します。無効になったキーは自動で破棄します。
                  </p>
                </div>

                <Button
                  type="submit"
                  size="lg"
                  disabled={isLoading || deviceSecret.trim().length === 0}
                  className="h-12 rounded-full px-6"
                >
                  {isLoading
                    ? (
                      <>
                        <RefreshCw className="animate-spin" />
                        接続中...
                      </>
                    )
                    : (
                      <>
                        <ShieldCheck />
                        接続する
                      </>
                    )}
                </Button>
              </form>
            </section>
          )}

          {visibleWorks && (
            <section className="space-y-8">
              {selectedWorkId
                ? (
                  <WorkDetailPanel deviceId={deviceId} lastSyncedAt={visibleWorks.lastSyncedAt} work={selectedWork} />
                )
                : (
                  <>
                    <div className="remote-library__toolbar">
                      <div>
                        <p className="text-[0.72rem] tracking-[0.28em] text-muted-foreground uppercase">Library</p>
                        <h1 className="mt-3 text-3xl font-semibold tracking-tight text-foreground sm:text-4xl">
                          同期された作品
                        </h1>
                      </div>

                      <div className="flex flex-wrap items-center gap-3">
                        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                          <div className="remote-library__stat">
                            <p className="remote-library__stat-label">Works</p>
                            <p className="remote-library__stat-value">{workCount}</p>
                          </div>
                          <div className="remote-library__stat">
                            <p className="remote-library__stat-label">Sync</p>
                            <p className="remote-library__stat-value text-sm sm:text-base">
                              {formatRelativeTime(visibleWorks.lastSyncedAt)}
                            </p>
                          </div>
                          <div className="remote-library__stat">
                            <p className="remote-library__stat-label">Updated</p>
                            <p className="remote-library__stat-value text-sm sm:text-base">
                              {formatExactDateTime(visibleWorks.lastSyncedAt)}
                            </p>
                          </div>
                        </div>

                        <ViewModeToggle viewMode={viewMode} onChange={setViewMode} />
                      </div>
                    </div>

                    {visibleWorks.works.length === 0
                      ? (
                        <div className="remote-gate">
                          <p className="text-lg font-semibold text-foreground">共有されている作品はまだありません</p>
                          <p className="mt-3 text-sm leading-7 text-muted-foreground">
                            次回同期が完了すると、ここにインストール済みタイトルが表示されます。
                          </p>
                        </div>
                      )
                      : (
                        <WorksPresentation deviceId={deviceId} viewMode={viewMode} works={visibleWorks.works} />
                      )}
                  </>
                )}
            </section>
          )}
        </div>
      </section>
    </main>
  )
}
