import type { DiagnosticProgress, SyncTrigger } from '@launcherg/shared'
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
  onProgress?: (progress: DiagnosticProgress, trigger: SyncTrigger) => void
}

export interface DlsiteRuntime {
  handleHookMessage: (event: MessageEvent<unknown>) => void
  handleUrlChange: (url: string) => void
  syncLatest: (trigger?: SyncTrigger) => Promise<DlsiteSyncResult>
}

export function createDlsiteRuntime(deps: DlsiteRuntimeDeps): DlsiteRuntime {
  const payloadCache = createUrlBoundPayloadCache<DlsiteWorksHookMessageData>(deps.initialUrl)
  let lastSyncedKey: string | null = null
  let syncTail: Promise<DlsiteSyncResult | undefined> = Promise.resolve(undefined)

  async function syncPayload(latestPayload: DlsiteWorksHookMessageData | null, trigger: SyncTrigger): Promise<DlsiteSyncResult> {
    const progress = (event: DiagnosticProgress) => deps.onProgress?.(event, trigger)
    if (!latestPayload) {
      progress({ stage: 'payload_missing' })
      return { success: false, message: MISSING_PAYLOAD_MESSAGE, error: MISSING_PAYLOAD_MESSAGE }
    }

    const payloadKey = buildDlsitePayloadKey(latestPayload)
    if (payloadKey === lastSyncedKey) {
      progress({ stage: 'sync_unchanged' })
      return { success: true, message: 'DLsite: 最新のAPIレスポンスは同期済みです' }
    }

    try {
      const extractedGames = extractDlsiteGamesFromApiResponse(latestPayload.payload)
      progress({ stage: 'games_extracted', count: extractedGames.length, detail: `file_types=${Array.from(new Set((latestPayload.payload.works || []).map(work => String(work.file_type || 'unknown')))).join(',')}` })
      if (extractedGames.length === 0) {
        progress({ stage: 'no_supported_games', count: 0 })
        return { success: false, message: 'DLsite: 同期対象のゲームが見つかりませんでした', error: '同期対象が見つかりませんでした' }
      }

      const processedGames = deps.processGames(extractedGames)
      progress({ stage: 'sync_requested', count: processedGames.length })
      await deps.syncDlsiteGames(processedGames)
      lastSyncedKey = payloadKey
      progress({ stage: 'sync_finished', count: processedGames.length })
      return { success: true, message: 'DLsite: 同期を実行しました' }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      log.error('Failed to sync DLsite payload:', error)
      progress({ stage: 'sync_failed', detail: errorMessage })
      deps.showErrorNotification('DLsite: 作品情報の同期に失敗しました')
      return { success: false, message: `DLsite: ${errorMessage}`, error: errorMessage }
    }
  }

  function syncLatest(trigger: SyncTrigger = 'manual'): Promise<DlsiteSyncResult> {
    // Capture each response before queueing; later pages must not overwrite it.
    const payload = payloadCache.get()
    const current = syncTail.catch(() => undefined).then(() => syncPayload(payload, trigger))
    syncTail = current
    return current
  }

  function handleHookMessage(event: MessageEvent<unknown>): void {
    if (!isTypedWindowMessage<DlsiteWorksHookMessageData>(event, DLSITE_HOOK_MESSAGE_SOURCE, DLSITE_WORKS_MESSAGE_TYPE))
      return

    payloadCache.set(event.data)
    deps.onProgress?.({ stage: 'payload_received', count: event.data.payload.works?.length || 0 }, 'auto')
    void syncLatest('auto')
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
