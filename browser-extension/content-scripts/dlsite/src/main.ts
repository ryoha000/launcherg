// DLsite用独立型抽出器のメインエントリーポイント

import { create, toJson } from '@bufbuild/protobuf'
import {
  addNotificationStyles,
  logger,
  sendExtensionRequest,
  setLogLevel,
  showNotification,
  waitForPageLoad,
} from '@launcherg/shared'
import { DlsiteGameSchema, DlsiteSyncGamesRequestSchema, ExtensionRequestSchema } from '@launcherg/shared/proto/extension_internal'

import { processGames } from './data-processor'
import { extractAllGames, shouldExtract } from './dom-extractor'

// グローバル状態管理
let isExtracting = false
let currentUrl = window.location.href
const log = logger('dlsite-extractor')
let pollingTimerId: number | null = null
let didInitialWait = false

// 抽出と同期を実行するメイン関数
async function extractAndSync(): Promise<void> {
  if (isExtracting) {
    log.debug('Already extracting, skipping')
    return
  }

  isExtracting = true

  try {
    // 初回のみページロード完了を待機
    if (!didInitialWait) {
      await waitForPageLoad(2000)
      didInitialWait = true
    }

    // ゲーム情報を抽出
    const games = extractAllGames()

    if (games.length === 0) {
      log.info('No games found')
      return
    }

    log.info(`Found ${games.length} games`)

    // DLsite特有の処理を適用
    const processedGames = processGames(games)

    // 同期リクエストを送信（DLsite専用）
    const dlsiteGames = processedGames.map(g => create(DlsiteGameSchema, {
      id: g.store_id || '',
      category: g.additional_data?.category || '',
    }))

    const request = create(ExtensionRequestSchema, {
      requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
      request: {
        case: 'syncDlsiteGames',
        value: create(DlsiteSyncGamesRequestSchema, {
          games: dlsiteGames,
          source: 'dlsite-extractor',
        }),
      },
    })

    try {
      const responseJson = await sendExtensionRequest(request, req => toJson(ExtensionRequestSchema, req))
      log.info('Sync successful:', responseJson)
    }
    catch (error) {
      log.error('Sync failed:', error)
    }
  }
  catch (error) {
    log.error('Extraction failed:', error)
    showNotification('DLsite: 作品情報の抽出に失敗しました', 'error')
  }
  finally {
    isExtracting = false
  }
}

// DLsite抽出器を初期化する関数
function initDLsiteExtractor(): void {
  const rootElement = document.getElementById('root')

  if (shouldExtract(window.location.hostname, rootElement)) {
    log.info('Target page detected - Starting extraction on DLsite')
    startPolling()
  }
  else {
    log.debug('Not a target page - skipping extraction')
    stopPolling()
  }
}

function startPolling(): void {
  if (pollingTimerId !== null)
    return
  // 即時実行 + 500ms間隔で実行
  void extractAndSync()
  pollingTimerId = window.setInterval(() => {
    void extractAndSync()
  }, 500)
  log.debug('Started polling every 500ms')
}

function stopPolling(): void {
  if (pollingTimerId !== null) {
    clearInterval(pollingTimerId)
    pollingTimerId = null
    log.debug('Stopped polling')
  }
}

// ページ変更を監視する関数（SPA対応）
function setupPageChangeObserver(): void {
  const observer = new MutationObserver(() => {
    if (window.location.href !== currentUrl) {
      currentUrl = window.location.href
      // URL変更時に再度チェック
      setTimeout(() => {
        // 初期待機をリセット
        didInitialWait = false
        initDLsiteExtractor()
      }, 2000)
    }
  })

  observer.observe(document.body, {
    childList: true,
    subtree: true,
  })
}

// メイン初期化処理
function main(): void {
  log.info('Script loaded')

  // CSSアニメーションを追加
  addNotificationStyles()

  // ページ変更の監視を設定
  setupPageChangeObserver()

  // 即座に抽出を開始（設定不要）
  setTimeout(() => {
    initDLsiteExtractor()
  }, 1000)
}

// スクリプトの実行
// 開発時はデバッグログを有効化
if ((import.meta as any).env?.MODE === 'development') {
  setLogLevel('debug')
}
main()

// バックグラウンド/ポップアップからのメッセージを受け取って同期を実行
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (
    message?.type === 'manual_sync_request'
    || message?.type === 'periodic_sync_check'
  ) {
    void extractAndSync()
      .then(() => sendResponse({ success: true, message: 'DLsite: 同期を実行しました' }))
      .catch((err: unknown) => {
        const errorMessage = err instanceof Error ? err.message : String(err)
        sendResponse({ success: false, error: errorMessage })
      })
    return true
  }
  return undefined
})
