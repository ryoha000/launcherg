import type { DeviceWorksListOutput } from '@server/shared/schema'
import type { FormEvent } from 'react'

import { api } from '@ui/api'
import { Alert, AlertDescription, AlertTitle } from '@ui/components/ui/alert'
import { Badge } from '@ui/components/ui/badge'
import { Button } from '@ui/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@ui/components/ui/card'
import { Input } from '@ui/components/ui/input'
import { Skeleton } from '@ui/components/ui/skeleton'
import {
  useEffect,
  useMemo,
  useState,
} from 'react'
import {
  HardDriveDownload,
  ImageIcon,
  KeyRound,
  Link2,
  RefreshCw,
  ShieldCheck,
  TriangleAlert,
} from 'lucide-react'

import {
  deleteStoredDeviceSecret,
  getAllStoredDeviceIds,
  getStoredDeviceSecret,
  setStoredDeviceSecret,
} from '@ui/lib/deviceSecretStore'

function getDeviceId(): string {
  const pathParts = window.location.pathname.split('/').filter(Boolean)
  return pathParts[0] ?? ''
}

function formatDateTime(value: string | null): string {
  if (!value) {
    return '未同期'
  }

  return new Date(value).toLocaleString('ja-JP')
}

export function App() {
  const deviceId = useMemo(() => getDeviceId(), [])
  const [deviceSecret, setDeviceSecret] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isRestoring, setIsRestoring] = useState(Boolean(deviceId))
  const [error, setError] = useState('')
  const [works, setWorks] = useState<DeviceWorksListOutput | null>(null)
  const [historyDevices, setHistoryDevices] = useState<string[]>([])

  useEffect(() => {
    let cancelled = false

    if (!deviceId) {
      void getAllStoredDeviceIds()
        .then((ids) => {
          if (!cancelled) {
            setHistoryDevices(ids)
          }
        })
        .catch((err) => {
          if (!cancelled) {
            setError(err instanceof Error ? err.message : String(err))
          }
        })
    }

    const restoreSession = async () => {
      if (!deviceId) {
        setIsRestoring(false)
        return
      }

      const storedSecret = await getStoredDeviceSecret(deviceId)
      if (!storedSecret || cancelled) {
        setIsRestoring(false)
        return
      }

      try {
        await api.createSession({
          deviceId,
          deviceSecret: storedSecret,
        })
        if (cancelled) {
          return
        }
        setDeviceSecret(storedSecret)
      }
      catch {
        if (!cancelled) {
          void deleteStoredDeviceSecret(deviceId)
          setError('保存済み secret が無効でした。再入力してください。')
        }
        return
      }
      finally {
        if (!cancelled) {
          setIsRestoring(false)
        }
      }

      try {
        const nextWorks = await api.listWorks(deviceId)
        if (cancelled) {
          return
        }

        setDeviceSecret(storedSecret)
        setWorks(nextWorks)
        setError('')
      }
      catch (cause) {
        if (!cancelled) {
          setError(cause instanceof Error ? cause.message : String(cause))
        }
      }
    }

    void restoreSession()

    return () => {
      cancelled = true
    }
  }, [deviceId])

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setIsLoading(true)
    setError('')
    const normalizedSecret = deviceSecret.trim()

    try {
      await api.createSession({
        deviceId,
        deviceSecret: normalizedSecret,
      })
      setDeviceSecret(normalizedSecret)
      void setStoredDeviceSecret(deviceId, normalizedSecret).catch(() => undefined)
      const nextWorks = await api.listWorks(deviceId)
      setWorks(nextWorks)
      setError('')
    }
    catch (cause) {
      void deleteStoredDeviceSecret(deviceId)
      setError(cause instanceof Error ? cause.message : String(cause))
    }
    finally {
      setIsLoading(false)
    }
  }

  return (
    <main className="min-h-screen overflow-hidden px-4 py-6 sm:px-6 sm:py-10">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-6">
        <section className="grid gap-6 lg:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">
          <Card className="border-white/40 bg-card/85 shadow-2xl shadow-black/8 backdrop-blur-sm">
            <CardHeader className="gap-4">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary">Launcherg Remote Share</Badge>
                <Badge variant="outline">{deviceId ? 'Device Ready' : 'URL Error'}</Badge>
              </div>
              <CardTitle className="text-3xl sm:text-4xl">
                インストール済みゲームを、
                <br />
                すぐに確認できる共有ビュー
              </CardTitle>
              <CardDescription className="max-w-2xl text-sm leading-6 sm:text-base">
                QR コードで開いた端末に対して共有用 secret を入力すると、同期済みの作品一覧を軽量な Web UI で参照できます。
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-3 sm:grid-cols-3">
              <div className="rounded-xl border bg-background/75 p-4">
                <ShieldCheck className="mb-3 text-muted-foreground" />
                <p className="text-sm font-medium">Session Cookie</p>
                <p className="mt-1 text-sm text-muted-foreground">保存済み secret があれば自動で再接続します。</p>
              </div>
              <div className="rounded-xl border bg-background/75 p-4">
                <HardDriveDownload className="mb-3 text-muted-foreground" />
                <p className="text-sm font-medium">Installed Works</p>
                <p className="mt-1 text-sm text-muted-foreground">同期済みタイトルを一覧でコンパクトに表示します。</p>
              </div>
              <div className="rounded-xl border bg-background/75 p-4">
                <ImageIcon className="mb-3 text-muted-foreground" />
                <p className="text-sm font-medium">Artwork Preview</p>
                <p className="mt-1 text-sm text-muted-foreground">サムネイル付きでブラウザからすぐ識別できます。</p>
              </div>
            </CardContent>
          </Card>

          <Card className="border-white/50 bg-card/90 shadow-xl shadow-black/8 backdrop-blur-sm">
            <CardHeader>
              <div className="flex items-center gap-2 text-muted-foreground">
                <KeyRound />
                <span className="text-sm font-medium">認証</span>
              </div>
              <CardTitle>共有セッション</CardTitle>
              <CardDescription>deviceId と共有 secret で作品一覧を復元します。</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4">
              {!deviceId && (
                <div className="flex flex-col gap-4">
                  <Alert variant="destructive">
                    <TriangleAlert />
                    <AlertTitle>deviceId が見つかりません</AlertTitle>
                    <AlertDescription>URL に deviceId が含まれていないため、共有先を特定できません。</AlertDescription>
                  </Alert>

                  {historyDevices.length > 0 && (
                    <div className="flex flex-col gap-3">
                      <div className="text-sm font-medium text-foreground">接続履歴</div>
                      <div className="grid gap-2">
                        {historyDevices.map((id) => (
                          <a
                            key={id}
                            href={`/${id}`}
                            className="flex items-center gap-2 rounded-xl border bg-muted/40 p-3 transition-colors hover:bg-muted/80"
                          >
                            <HardDriveDownload className="h-5 w-5 text-muted-foreground" />
                            <span className="font-mono text-sm">{id}</span>
                          </a>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {error && (
                <Alert variant="destructive">
                  <TriangleAlert />
                  <AlertTitle>認証に失敗しました</AlertTitle>
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              {deviceId && !works && isRestoring && (
                <div className="flex flex-col gap-3">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <RefreshCw className="animate-spin" />
                    保存済みの secret を確認中...
                  </div>
                  <Skeleton className="h-11 w-full rounded-xl" />
                  <Skeleton className="h-11 w-full rounded-xl" />
                  <Skeleton className="h-24 w-full rounded-2xl" />
                </div>
              )}

              {deviceId && !works && !isRestoring && (
                <form className="flex flex-col gap-4" onSubmit={submit}>
                  <div className="flex flex-col gap-2">
                    <label className="text-sm font-medium text-foreground" htmlFor="device-id">
                      deviceId
                    </label>
                    <Input
                      id="device-id"
                      readOnly
                      value={deviceId}
                      className="h-11 rounded-xl bg-muted/60"
                    />
                  </div>
                  <div className="flex flex-col gap-2">
                    <label className="text-sm font-medium text-foreground" htmlFor="device-secret">
                      deviceSecret
                    </label>
                    <Input
                      id="device-secret"
                      value={deviceSecret}
                      onChange={event => setDeviceSecret(event.target.value)}
                      placeholder="共有 secret を入力"
                      type="password"
                      autoComplete="off"
                      className="h-11 rounded-xl bg-background"
                    />
                  </div>
                  <Button
                    type="submit"
                    size="lg"
                    disabled={isLoading || deviceSecret.trim().length === 0}
                    className="h-11 rounded-xl"
                  >
                    {isLoading ? '確認中...' : '一覧を表示'}
                  </Button>
                </form>
              )}

              {works && (
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="rounded-xl border bg-background/80 p-4">
                    <p className="text-sm text-muted-foreground">共有タイトル数</p>
                    <p className="mt-1 text-3xl font-semibold">{works.works.length}</p>
                  </div>
                  <div className="rounded-xl border bg-background/80 p-4">
                    <p className="text-sm text-muted-foreground">最終同期</p>
                    <p className="mt-1 text-sm font-medium leading-6">{formatDateTime(works.lastSyncedAt)}</p>
                  </div>
                </div>
              )}
            </CardContent>
            <CardFooter className="justify-between gap-3 text-xs text-muted-foreground">
              <div className="flex items-center gap-2">
                <Link2 />
                <span>同一オリジンの API に Cookie 付きで接続</span>
              </div>
            </CardFooter>
          </Card>
        </section>

        {works && (
          <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {works.works.length === 0 && (
              <Card className="border-dashed bg-card/85 md:col-span-2 xl:col-span-3">
                <CardHeader>
                  <CardTitle>共有されている作品はまだありません</CardTitle>
                  <CardDescription>次回同期が完了すると、ここにインストール済みタイトルが表示されます。</CardDescription>
                </CardHeader>
              </Card>
            )}

            {works.works.map(work => (
              <Card key={work.workId} className="overflow-hidden bg-card/90 shadow-lg shadow-black/5">
                {work.imageUrl
                  ? (
                      <img
                        alt={work.title}
                        className="aspect-[16/9] w-full object-cover"
                        src={work.imageUrl}
                      />
                    )
                  : (
                      <div className="flex aspect-[16/9] items-center justify-center bg-muted text-muted-foreground">
                        <ImageIcon />
                      </div>
                    )}
                <CardHeader>
                  <CardTitle className="line-clamp-2 text-lg">{work.title}</CardTitle>
                  <CardDescription className="break-all">{work.workId}</CardDescription>
                </CardHeader>
                <CardContent className="flex items-center justify-between gap-3">
                  <Badge variant="outline">Installed</Badge>
                  <span className="text-xs text-muted-foreground">Remote Share</span>
                </CardContent>
              </Card>
            ))}
          </section>
        )}
      </div>
    </main>
  )
}
