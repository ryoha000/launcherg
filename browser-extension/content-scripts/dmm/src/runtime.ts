import type { DiagnosticProgress, SyncTrigger } from '@launcherg/shared'
import type { DmmApiExtractedGame, DmmLibraryHookMessageData } from './api'
import type { DmmExtractedGame } from './types'
import {
  createUrlBoundPayloadCache,
  isTypedWindowMessage,
  logger,
} from '@launcherg/shared'
import {
  buildDmmPayloadKey,
  DMM_HOOK_MESSAGE_SOURCE,
  DMM_LIBRARY_MESSAGE_TYPE,
  extractDmmGamesFromApiResponse,
  splitDmmApiGames,
} from './api'

const log = logger('dmm-runtime')
const MISSING_PAYLOAD_MESSAGE = 'DMM: APIレスポンス未取得。ページを再読み込みしてください'

export interface DmmSyncResult {
  success: boolean
  message: string
  error?: string
}

export interface DmmRuntimeDeps {
  initialUrl: string
  processPacks: (packGames: DmmApiExtractedGame[]) => Promise<DmmExtractedGame[]>
  syncDmmGames: (games: DmmExtractedGame[]) => Promise<void>
  showErrorNotification: (message: string) => void
  onProgress?: (progress: DiagnosticProgress, trigger: SyncTrigger) => void
}

export interface DmmRuntime {
  handleHookMessage: (event: MessageEvent<unknown>) => void
  handleUrlChange: (url: string) => void
  syncLatest: (trigger?: SyncTrigger) => Promise<DmmSyncResult>
  getLatestPayload: () => DmmLibraryHookMessageData | null
}

export function createDmmRuntime(deps: DmmRuntimeDeps): DmmRuntime {
  const payloadCache = createUrlBoundPayloadCache<DmmLibraryHookMessageData>(deps.initialUrl)
  let lastSyncedKey: string | null = null
  let syncTail: Promise<DmmSyncResult | undefined> = Promise.resolve(undefined)

  async function syncPayload(latestPayload: DmmLibraryHookMessageData | null, trigger: SyncTrigger): Promise<DmmSyncResult> {
    const progress = (event: DiagnosticProgress) => deps.onProgress?.(event, trigger)
    if (!latestPayload) {
      progress({ stage: 'payload_missing' })
      return { success: false, message: MISSING_PAYLOAD_MESSAGE, error: MISSING_PAYLOAD_MESSAGE }
    }

    const payloadKey = buildDmmPayloadKey(latestPayload)
    if (payloadKey === lastSyncedKey) {
      progress({ stage: 'sync_unchanged' })
      return { success: true, message: 'DMM: 最新のAPIレスポンスは同期済みです' }
    }

    log.info('syncLatest start', { requestUrl: latestPayload.requestUrl })
    try {
      const extractedGames = extractDmmGamesFromApiResponse(latestPayload.payload)
      progress({ stage: 'games_extracted', count: extractedGames.length })
      log.debug('extracted games', { count: extractedGames.length })
      if (extractedGames.length === 0) {
        progress({ stage: 'no_supported_games', count: 0 })
        return { success: false, message: 'DMM: 同期対象のゲームが見つかりませんでした', error: '同期対象が見つかりませんでした' }
      }

      const { normalGames, packGames } = splitDmmApiGames(extractedGames)
      log.info('split pack games', { normalCount: normalGames.length, packCount: packGames.length })
      const expandedPackGames = packGames.length > 0
        ? await deps.processPacks(packGames)
        : []

      log.info('expanded pack games', { childCount: expandedPackGames.length })
      const games = [...normalGames, ...packGames, ...expandedPackGames]
      progress({ stage: 'sync_requested', count: games.length })
      await deps.syncDmmGames(games)
      lastSyncedKey = payloadKey
      progress({ stage: 'sync_finished', count: games.length })
      return { success: true, message: 'DMM: 同期を実行しました' }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      log.error('Failed to sync DMM payload:', error)
      progress({ stage: 'sync_failed', detail: errorMessage })
      deps.showErrorNotification('DMM: ゲーム情報の同期に失敗しました')
      return { success: false, message: `DMM: ${errorMessage}`, error: errorMessage }
    }
  }

  function syncLatest(trigger: SyncTrigger = 'manual'): Promise<DmmSyncResult> {
    // Capture each response before queueing; later pages must not overwrite it.
    const payload = payloadCache.get()
    const current = syncTail.catch(() => undefined).then(() => syncPayload(payload, trigger))
    syncTail = current
    return current
  }

  function handleHookMessage(event: MessageEvent<unknown>): void {
    if (!isTypedWindowMessage<DmmLibraryHookMessageData>(event, DMM_HOOK_MESSAGE_SOURCE, DMM_LIBRARY_MESSAGE_TYPE))
      return

    log.debug('hook message received', { requestUrl: event.data.requestUrl })
    payloadCache.set(event.data)
    deps.onProgress?.({ stage: 'payload_received' }, 'auto')
    void syncLatest('auto')
  }

  function handleUrlChange(url: string): void {
    if (payloadCache.resetIfUrlChanged(url)) {
      log.debug('url changed - reset sync cache', { url })
      lastSyncedKey = null
    }
  }

  return {
    handleHookMessage,
    handleUrlChange,
    syncLatest,
    getLatestPayload: () => payloadCache.get(),
  }
}
