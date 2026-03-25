import type { DeviceWorksListOutput } from '@server/shared/schema'
import type { FormEvent } from 'react'

import { api } from '@ui/api'
import {
  useEffect,
  useMemo,
  useState,
} from 'react'

import {
  deleteStoredDeviceSecret,
  getStoredDeviceSecret,
  setStoredDeviceSecret,
} from '@ui/lib/deviceSecretStore'

function getDeviceId(): string {
  const pathParts = window.location.pathname.split('/').filter(Boolean)
  return pathParts[0] ?? ''
}

export function App() {
  const deviceId = useMemo(() => getDeviceId(), [])
  const [deviceSecret, setDeviceSecret] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isRestoring, setIsRestoring] = useState(Boolean(deviceId))
  const [error, setError] = useState('')
  const [works, setWorks] = useState<DeviceWorksListOutput | null>(null)

  useEffect(() => {
    let cancelled = false

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
    }
    catch (cause) {
      void deleteStoredDeviceSecret(deviceId)
      setError(cause instanceof Error ? cause.message : String(cause))
      return
    }

    void setStoredDeviceSecret(deviceId, normalizedSecret).catch(() => undefined)

    try {
      const nextWorks = await api.listWorks(deviceId)
      setWorks(nextWorks)
      setError('')
    }
    catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause))
    }
    finally {
      setIsLoading(false)
    }
  }

  return (
    <main className="page">
      <section className="panel">
        <header className="panel-header">
          <p className="eyebrow">Launcherg Remote Share</p>
          <h1>インストール済みゲーム</h1>
          <p className="description">
            QR コードで開いたデバイスに対して、共有用 secret を入力すると一覧を確認できます。
          </p>
        </header>

        {!deviceId && (
          <div className="empty-state">
            deviceId が URL に含まれていません。
          </div>
        )}

        {deviceId && !works && !isRestoring && (
          <form className="auth-form" onSubmit={submit}>
            <label className="field">
              <span>deviceId</span>
              <input value={deviceId} readOnly />
            </label>
            <label className="field">
              <span>deviceSecret</span>
              <input
                value={deviceSecret}
                onChange={event => setDeviceSecret(event.target.value)}
                placeholder="共有 secret を入力"
                type="password"
                autoComplete="off"
              />
            </label>
            {error && <p className="error">{error}</p>}
            <button type="submit" disabled={isLoading || deviceSecret.trim().length === 0}>
              {isLoading ? '確認中...' : '一覧を表示'}
            </button>
          </form>
        )}

        {deviceId && !works && isRestoring && (
          <div className="empty-state">
            保存済みの secret を確認中...
          </div>
        )}

        {works && (
          <section className="works">
            <div className="summary">
              <div>
                <strong>{works.works.length}</strong>
                {' '}
                件
              </div>
              <div>
                最終同期:
                {' '}
                {works.lastSyncedAt ? new Date(works.lastSyncedAt).toLocaleString('ja-JP') : '未同期'}
              </div>
            </div>

            {works.works.length === 0 && (
              <div className="empty-state">共有されている作品はまだありません。</div>
            )}

            <div className="work-list">
              {works.works.map(work => (
                <article className="work-card" key={work.workId}>
                  {work.imageUrl
                    ? <img alt={work.title} className="thumbnail" src={work.imageUrl} />
                    : <div className="thumbnail placeholder">NO IMAGE</div>}
                  <div className="work-body">
                    <h2>{work.title}</h2>
                  </div>
                </article>
              ))}
            </div>
          </section>
        )}
      </section>
    </main>
  )
}
