import type { DeviceWorksListOutput } from '@server/shared/schema'

export interface StoredDeviceHistory {
  deviceId: string
  lastUsedAt?: string
}

export type ViewMode = 'masonry' | 'list'

const relativeTimeFormatter = new Intl.RelativeTimeFormat('ja-JP', {
  numeric: 'auto',
})

const numberFormatter = new Intl.NumberFormat('ja-JP')

export function getDeviceId(): string {
  const pathParts = window.location.pathname.split('/').filter(Boolean)
  return pathParts[0] ?? ''
}

export function getSelectedWorkId(): string | null {
  const pathParts = window.location.pathname.split('/').filter(Boolean)
  if (pathParts.length >= 3 && pathParts[1] === 'works') {
    return decodeURIComponent(pathParts[2] ?? '')
  }

  return null
}

export function getWorkDetailPath(deviceId: string, workId: string): string {
  return `/${encodeURIComponent(deviceId)}/works/${encodeURIComponent(workId)}`
}

export function getLibraryPath(deviceId: string): string {
  return `/${encodeURIComponent(deviceId)}`
}

export function dedupeBy<T>(items: T[], getKey: (item: T) => string): T[] {
  const seen = new Set<string>()

  return items.filter((item) => {
    const key = getKey(item)
    if (seen.has(key)) {
      return false
    }

    seen.add(key)
    return true
  })
}

export function formatCount(value: number): string {
  return numberFormatter.format(value)
}

export function formatShortDeviceId(value: string): string {
  if (value.length <= 16) {
    return value
  }

  return `${value.slice(0, 8)}…${value.slice(-6)}`
}

export function formatExactDateTime(value: string | null | undefined): string {
  if (!value) {
    return '未同期'
  }

  return new Date(value).toLocaleString('ja-JP', {
    dateStyle: 'medium',
    timeStyle: 'short',
  })
}

export function formatRelativeTime(value: string | null | undefined): string {
  if (!value) {
    return '未同期'
  }

  const time = new Date(value).getTime()
  if (Number.isNaN(time)) {
    return '未同期'
  }

  const diffSeconds = Math.round((time - Date.now()) / 1000)
  const absSeconds = Math.abs(diffSeconds)

  if (absSeconds < 60) {
    return relativeTimeFormatter.format(diffSeconds, 'second')
  }

  const diffMinutes = Math.round(diffSeconds / 60)
  const absMinutes = Math.abs(diffMinutes)
  if (absMinutes < 60) {
    return relativeTimeFormatter.format(diffMinutes, 'minute')
  }

  const diffHours = Math.round(diffMinutes / 60)
  const absHours = Math.abs(diffHours)
  if (absHours < 24) {
    return relativeTimeFormatter.format(diffHours, 'hour')
  }

  const diffDays = Math.round(diffHours / 24)
  if (Math.abs(diffDays) < 7) {
    return relativeTimeFormatter.format(diffDays, 'day')
  }

  return formatExactDateTime(value)
}

export function getFriendlyErrorMessage(cause: unknown, hasDeviceId: boolean): string {
  const rawMessage = cause instanceof Error ? cause.message : String(cause)

  if (!hasDeviceId) {
    return 'PC版アプリで開いたURLか、QRコードを読み取ったあとに表示されるページです。保存済みの接続先があれば、下から選んでください。'
  }

  if (/deviceSecret is invalid|secret.*invalid/i.test(rawMessage)) {
    return '共有キーが一致しません。PC版アプリで共有キーを確認して、もう一度入力してください。'
  }

  if (/device not found|not found/i.test(rawMessage)) {
    return 'このURLは使えません。PC版アプリでURLを作り直してから、もう一度開いてください。'
  }

  if (/IndexedDB/i.test(rawMessage)) {
    return 'このブラウザでは接続情報を保存できません。別のブラウザでお試しください。'
  }

  return '接続できませんでした。通信状態を確認して、もう一度お試しください。'
}

export function getLandingHeroTitle({
  hasDeviceId,
  visibleWorks,
  historyCount,
}: {
  hasDeviceId: boolean
  visibleWorks: DeviceWorksListOutput | null
  historyCount: number
}): string {
  if (visibleWorks) {
    return 'PCにあるゲームへ、スマホからそのまま戻れる。'
  }

  if (hasDeviceId) {
    return 'この場で接続を終わらせて、ゲーム一覧へ進む。'
  }

  if (historyCount > 0) {
    return '前に使ったPCへ、最短で戻るための入口。'
  }

  return 'PCで共有したゲーム一覧を、スマホから開く。'
}

export function getLandingHeroDescription({
  hasDeviceId,
  visibleWorks,
  historyCount,
}: {
  hasDeviceId: boolean
  visibleWorks: DeviceWorksListOutput | null
  historyCount: number
}): string {
  if (visibleWorks) {
    return '接続は完了しています。同期済みの作品を確認して、そのまま次の操作へ進めます。'
  }

  if (hasDeviceId) {
    return '共有キーを入力すると、このブラウザに接続先を保存します。次回からは同じ端末でそのまま開けます。'
  }

  if (historyCount > 0) {
    return '保存済みの接続先があれば下からすぐ再接続できます。新しい端末はPC版アプリで表示したURLまたはQRコードから開いてください。'
  }

  return 'このページはPC版 Launcherg が発行したURLから開きます。接続が保存されると、次回からは選ぶだけで戻れます。'
}
