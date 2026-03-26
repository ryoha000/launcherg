import type { DeviceWorksListOutput } from '@server/shared/schema'
import type { FormEvent } from 'react'

import { api } from '@ui/api'
import { DeviceWorkspace } from '@ui/components/remote-share/DeviceWorkspace'
import { LandingPage } from '@ui/components/remote-share/LandingPage'
import {
  deleteStoredDeviceSecret,
  getAllStoredDeviceHistories,
  getStoredDeviceSecret,
  setStoredDeviceSecret,
} from '@ui/lib/deviceSecretStore'
import {
  dedupeBy,
  getDeviceId,
  getFriendlyErrorMessage,
  getLandingHeroDescription,
  getLandingHeroTitle,
  type StoredDeviceHistory,
  type ViewMode,
} from '@ui/lib/remoteShare'
import { useEffect, useState } from 'react'

export function App() {
  const deviceId = getDeviceId()
  const hasDeviceId = deviceId.length > 0
  const [deviceSecret, setDeviceSecret] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isRestoring, setIsRestoring] = useState(hasDeviceId)
  const [isLoadingHistory, setIsLoadingHistory] = useState(!hasDeviceId)
  const [error, setError] = useState('')
  const [works, setWorks] = useState<DeviceWorksListOutput | null>(null)
  const [historyDevices, setHistoryDevices] = useState<StoredDeviceHistory[]>([])
  const [viewMode, setViewMode] = useState<ViewMode>('masonry')

  useEffect(() => {
    const nextViewMode = window.localStorage.getItem('remote-share:view-mode')
    switch (nextViewMode) {
      case 'list':
      case 'masonry':
        setViewMode(nextViewMode)
        break
      default:
        break
    }
  }, [])

  useEffect(() => {
    window.localStorage.setItem('remote-share:view-mode', viewMode)
  }, [viewMode])

  useEffect(() => {
    let cancelled = false

    if (!hasDeviceId) {
      setIsLoadingHistory(true)
      void getAllStoredDeviceHistories()
        .then((records) => {
          if (cancelled) {
            return
          }

          const nextRecords = dedupeBy(
            [...records].sort((left, right) => {
              const leftAt = Date.parse(left.lastUsedAt ?? '') || 0
              const rightAt = Date.parse(right.lastUsedAt ?? '') || 0
              return rightAt - leftAt
            }),
            history => history.deviceId,
          )

          setHistoryDevices(nextRecords)
        })
        .catch(() => {
          if (!cancelled) {
            setError('接続履歴を読み込めませんでした。ブラウザの保存データを確認してから、もう一度お試しください。')
          }
        })
        .finally(() => {
          if (!cancelled) {
            setIsLoadingHistory(false)
          }
        })
    }

    const restoreSession = async () => {
      if (!hasDeviceId) {
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

        void setStoredDeviceSecret(deviceId, storedSecret).catch(() => undefined)
        setDeviceSecret(storedSecret)
        const nextWorks = await api.listWorks(deviceId)

        if (cancelled) {
          return
        }

        setWorks({
          ...nextWorks,
          works: dedupeBy(nextWorks.works, work => work.workId),
        })
        setError('')
      }
      catch (cause) {
        if (!cancelled) {
          void deleteStoredDeviceSecret(deviceId)
          setDeviceSecret('')
          setError(getFriendlyErrorMessage(cause, hasDeviceId))
        }
      }
      finally {
        if (!cancelled) {
          setIsRestoring(false)
        }
      }
    }

    void restoreSession()

    return () => {
      cancelled = true
    }
  }, [deviceId, hasDeviceId])

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    if (!hasDeviceId) {
      return
    }

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
      setWorks({
        ...nextWorks,
        works: dedupeBy(nextWorks.works, work => work.workId),
      })
      setError('')
    }
    catch (cause) {
      void deleteStoredDeviceSecret(deviceId)
      setError(getFriendlyErrorMessage(cause, hasDeviceId))
    }
    finally {
      setIsLoading(false)
    }
  }

  const sortedHistory = dedupeBy([...historyDevices], history => history.deviceId)
  const visibleWorks = works
    ? {
      ...works,
      works: dedupeBy(works.works, work => work.workId),
    }
    : null

  const latestHistoryAt = sortedHistory[0]?.lastUsedAt ?? null

  if (hasDeviceId) {
    return (
      <DeviceWorkspace
        deviceId={deviceId}
        deviceSecret={deviceSecret}
        error={error}
        isLoading={isLoading}
        isRestoring={isRestoring}
        onSecretChange={setDeviceSecret}
        onSubmit={submit}
        setViewMode={setViewMode}
        viewMode={viewMode}
        visibleWorks={visibleWorks}
      />
    )
  }

  return (
    <LandingPage
      error={error}
      heroDescription={getLandingHeroDescription({
        hasDeviceId,
        visibleWorks,
        historyCount: sortedHistory.length,
      })}
      heroTitle={getLandingHeroTitle({
        hasDeviceId,
        visibleWorks,
        historyCount: sortedHistory.length,
      })}
      historyDevices={sortedHistory}
      isLoadingHistory={isLoadingHistory}
      latestHistoryAt={latestHistoryAt}
      primaryActionLabel={sortedHistory.length > 0 ? '保存済みの接続先を見る' : '接続方法を確認する'}
    />
  )
}
