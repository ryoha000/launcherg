import type { DlsiteWorksHookMessageData } from './api'
import type { DlsiteExtractedGame } from './types'
import {
  createUrlBoundPayloadCache,
  isTypedWindowMessage,
  logger,
} from '@launcherg/shared'
import {
  buildDlsitePayloadKey,
  DLSITE_HOOK_MESSAGE_SOURCE,
  DLSITE_WORKS_MESSAGE_TYPE,
  extractDlsiteGamesFromApiResponse,
} from './api'

const log = logger('dlsite-runtime')
const MISSING_PAYLOAD_MESSAGE = 'DLsite: APIレスポンス未取得。ページを再読み込みしてください'

export interface DlsiteSyncResult {
  success: boolean
  message: string
  error?: string
}

export interface DlsiteRuntimeDeps {
  initialUrl: string
  processGames: (games: DlsiteExtractedGame[]) => DlsiteExtractedGame[]
  syncDlsiteGames: (games: DlsiteExtractedGame[]) => Promise<void>
  showErrorNotification: (message: string) => void
}

export interface DlsiteRuntime {
  handleHookMessage: (event: MessageEvent<unknown>) => void
  handleUrlChange: (url: string) => void
  syncLatest: () => Promise<DlsiteSyncResult>
}

export function createDlsiteRuntime(deps: DlsiteRuntimeDeps): DlsiteRuntime {
  const payloadCache = createUrlBoundPayloadCache<DlsiteWorksHookMessageData>(deps.initialUrl)
  let lastSyncedKey: string | null = null
  let isSyncing = false

  async function syncLatest(): Promise<DlsiteSyncResult> {
    const latestPayload = payloadCache.get()
    if (!latestPayload) {
      return { success: false, message: MISSING_PAYLOAD_MESSAGE, error: MISSING_PAYLOAD_MESSAGE }
    }

    if (isSyncing) {
      return { success: false, message: 'DLsite: 同期を実行中です', error: '同期を実行中です' }
    }

    const payloadKey = buildDlsitePayloadKey(latestPayload)
    if (payloadKey === lastSyncedKey) {
      return { success: true, message: 'DLsite: 最新のAPIレスポンスは同期済みです' }
    }

    isSyncing = true
    try {
      const extractedGames = extractDlsiteGamesFromApiResponse(latestPayload.payload)
      if (extractedGames.length === 0) {
        return { success: false, message: 'DLsite: 同期対象のゲームが見つかりませんでした', error: '同期対象が見つかりませんでした' }
      }

      const processedGames = deps.processGames(extractedGames)
      await deps.syncDlsiteGames(processedGames)
      lastSyncedKey = payloadKey
      return { success: true, message: 'DLsite: 同期を実行しました' }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      log.error('Failed to sync DLsite payload:', error)
      deps.showErrorNotification('DLsite: 作品情報の同期に失敗しました')
      return { success: false, message: `DLsite: ${errorMessage}`, error: errorMessage }
    }
    finally {
      isSyncing = false
    }
  }

  function handleHookMessage(event: MessageEvent<unknown>): void {
    if (!isTypedWindowMessage<DlsiteWorksHookMessageData>(event, DLSITE_HOOK_MESSAGE_SOURCE, DLSITE_WORKS_MESSAGE_TYPE))
      return

    payloadCache.set(event.data)
    void syncLatest()
  }

  function handleUrlChange(url: string): void {
    if (payloadCache.resetIfUrlChanged(url)) {
      lastSyncedKey = null
    }
  }

  return {
    handleHookMessage,
    handleUrlChange,
    syncLatest,
  }
}
