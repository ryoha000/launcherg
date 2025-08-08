// DLsite用独立型抽出器のメインエントリーポイント

import {
  addNotificationStyles,
  sendSyncRequest,
  setLogLevel,
  showNotification,
  waitForPageLoad,
} from '@launcherg/shared'
import { processGames } from './data-processor'
import { extractAllGames, shouldExtract } from './dom-extractor'

// グローバル状態管理
let isExtracting = false
let currentUrl = window.location.href

// 抽出と同期を実行するメイン関数
async function extractAndSync(): Promise<void> {
  if (isExtracting) {
    console.log('[DLsite Extractor] Already extracting, skipping')
    return
  }

  isExtracting = true

  try {
    // ページが完全に読み込まれるまで待機（DLsiteは動的コンテンツが多いので長めの待機）
    await waitForPageLoad(2000)

    // ゲーム情報を抽出
    const games = extractAllGames()

    if (games.length === 0) {
      console.log('[DLsite Extractor] No games found')
      return
    }

    console.log(`[DLsite Extractor] Found ${games.length} games`)

    // DLsite特有の処理を適用
    const processedGames = processGames(games)

    // 同期リクエストを送信
    sendSyncRequest(
      'DLSite',
      processedGames,
      'dlsite-extractor',
      (response) => {
        console.log('[DLsite Extractor] Sync successful:', response)
        showNotification(
          `DLsite: ${processedGames.length}個の作品を同期しました`,
        )
      },
      (error) => {
        console.error('[DLsite Extractor] Sync failed:', error)
        showNotification('DLsite: 同期に失敗しました', 'error')
      },
    )
  }
  catch (error) {
    console.error('[DLsite Extractor] Extraction failed:', error)
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
    console.log(
      '[DLsite Extractor] Target page detected - Starting extraction on DLsite',
    )
    extractAndSync()
  }
  else {
    console.log('[DLsite Extractor] Not a target page - skipping extraction')
  }
}

// ページ変更を監視する関数（SPA対応）
function setupPageChangeObserver(): void {
  const observer = new MutationObserver(() => {
    if (window.location.href !== currentUrl) {
      currentUrl = window.location.href
      // URL変更時に再度チェック
      setTimeout(() => {
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
  console.log('[DLsite Extractor] Script loaded')

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
