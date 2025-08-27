import type { DmmExtractedGame } from './types'
import {
  addNotificationStyles,
  logger,
  showInPageNotification,
  waitForPageLoad,
} from '@launcherg/shared'
import { extractAllGames, shouldExtract } from './dom-extractor'
import { fetchPackIds, fetchPackParentMap, processPacks, syncDmmGames } from './orchestrator'

let isExtracting = false
let currentUrl = window.location.href
const log = logger('dmm-extractor')
const lastSyncedUrls = new Set<string>()

async function extractAndSync(): Promise<void> {
  if (isExtracting) {
    log.debug('Already extracting, skipping')
    return
  }

  isExtracting = true

  try {
    const games = extractAllGames()
    if (games.length === 0) {
      log.info('No games found')
      return
    }
    // 1) パックIDの取得
    const packSet = await fetchPackIds()
    // 2) パック要素はfetchで詳細取得し、配列で受け取る
    const packOnly = games.filter(g => packSet.has(g.storeId))
    const normalGames = games.filter(g => !packSet.has(g.storeId))
    let packGames: DmmExtractedGame[] = []
    if (packOnly.length > 0) {
      const parentMap = await fetchPackParentMap()
      packGames = await processPacks(new Set(packOnly.map(g => g.storeId)), parentMap)
    }

    // 3) パック配下ゲームと通常ゲームを結合し、一度だけ同期
    const allGames: DmmExtractedGame[] = [...normalGames, ...packGames]
    await syncDmmGames(allGames)
  }
  catch (error) {
    log.error('Extraction failed:', error)
    showInPageNotification('DMM: ゲーム情報の抽出に失敗しました', 'error')
  }
  finally {
    isExtracting = false
  }
}

async function initDmmExtractor(): Promise<void> {
  const rootElement = document.getElementById('mylibrary')
  if (!shouldExtract(window.location.hostname, rootElement)) {
    log.debug('Not a target page - skipping extraction')
    return
  }

  const url = window.location.href
  if (lastSyncedUrls.has(url)) {
    log.debug('Already synced for this URL, skipping')
    return
  }

  await waitForPageLoad(2000)
  lastSyncedUrls.add(url)
  log.info('Target page detected - Extracting once on DMM')
  void extractAndSync()
}

function setupPageChangeObserver(): void {
  const observer = new MutationObserver(() => {
    if (window.location.href !== currentUrl) {
      currentUrl = window.location.href
      void initDmmExtractor()
    }
  })
  observer.observe(document.body, { childList: true, subtree: true })
}

function main(): void {
  log.info('Script loaded')
  addNotificationStyles()
  setupPageChangeObserver()
  setTimeout(() => {
    void initDmmExtractor()
  }, 1000)
}

main()

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === 'manual_sync_request' || message?.type === 'periodic_sync_check') {
    void extractAndSync()
      .then(() => sendResponse({ success: true, message: 'DMM: 同期を実行しました' }))
      .catch((err: unknown) => {
        const errorMessage = err instanceof Error ? err.message : String(err)
        sendResponse({ success: false, error: errorMessage })
      })
    return true
  }
  return undefined
})
