import type { DmmLibraryHookMessageData } from './api'
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
  fetchPackParentMap: () => Promise<Map<string, number>>
  processPacks: (packStoreIds: Set<string>, parentMap?: Map<string, number>) => Promise<DmmExtractedGame[]>
  syncDmmGames: (games: DmmExtractedGame[]) => Promise<void>
  showErrorNotification: (message: string) => void
}

export interface DmmRuntime {
  handleHookMessage: (event: MessageEvent<unknown>) => void
  handleUrlChange: (url: string) => void
  syncLatest: () => Promise<DmmSyncResult>
  getLatestPayload: () => DmmLibraryHookMessageData | null
}

export function createDmmRuntime(deps: DmmRuntimeDeps): DmmRuntime {
  const payloadCache = createUrlBoundPayloadCache<DmmLibraryHookMessageData>(deps.initialUrl)
  let lastSyncedKey: string | null = null
  let isSyncing = false

  async function syncLatest(): Promise<DmmSyncResult> {
    const latestPayload = payloadCache.get()
    if (!latestPayload) {
      return { success: false, message: MISSING_PAYLOAD_MESSAGE, error: MISSING_PAYLOAD_MESSAGE }
    }

    if (isSyncing) {
      return { success: false, message: 'DMM: 同期を実行中です', error: '同期を実行中です' }
    }

    const payloadKey = buildDmmPayloadKey(latestPayload)
    if (payloadKey === lastSyncedKey) {
      return { success: true, message: 'DMM: 最新のAPIレスポンスは同期済みです' }
    }

    isSyncing = true
    try {
      const extractedGames = extractDmmGamesFromApiResponse(latestPayload.payload)
      if (extractedGames.length === 0) {
        return { success: false, message: 'DMM: 同期対象のゲームが見つかりませんでした', error: '同期対象が見つかりませんでした' }
      }

      const { normalGames, packGames } = splitDmmApiGames(extractedGames)
      const packParentMap = packGames.length > 0 ? await deps.fetchPackParentMap() : undefined
      const expandedPackGames = packGames.length > 0
        ? await deps.processPacks(new Set(packGames.map(game => game.storeId)), packParentMap)
        : []

      await deps.syncDmmGames([...normalGames, ...expandedPackGames])
      lastSyncedKey = payloadKey
      return { success: true, message: 'DMM: 同期を実行しました' }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error)
      log.error('Failed to sync DMM payload:', error)
      deps.showErrorNotification('DMM: ゲーム情報の同期に失敗しました')
      return { success: false, message: `DMM: ${errorMessage}`, error: errorMessage }
    }
    finally {
      isSyncing = false
    }
  }

  function handleHookMessage(event: MessageEvent<unknown>): void {
    if (!isTypedWindowMessage<DmmLibraryHookMessageData>(event, DMM_HOOK_MESSAGE_SOURCE, DMM_LIBRARY_MESSAGE_TYPE))
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
    getLatestPayload: () => payloadCache.get(),
  }
}
