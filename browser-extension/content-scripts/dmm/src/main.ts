import type { DmmExtractedGame } from './types'
import {
  addNotificationStyles,
  logger,
  showInPageNotification,
  waitForPageLoad,
} from '@launcherg/shared'
import { extractAllGames, shouldExtract } from './dom-extractor'
import { initLaunchergDownloadOnceForUrl } from './download'
import { fetchOmitWorks, processPacks, syncDmmGames } from './orchestrator'

const EXTRACTION_RETRY_WINDOW_MS = 30_000

let isExtracting = false
let currentUrl = window.location.href
let extractionTimer: ReturnType<typeof setTimeout> | null = null
const extractionStartedAtByUrl = new Map<string, number>()
const log = logger('dmm-extractor')
const lastSyncedUrls = new Set<string>()
const downloadTriggeredForUrl = new Set<string>()
const isMarked = (url: string) => downloadTriggeredForUrl.has(url)
const mark = (url: string) => void downloadTriggeredForUrl.add(url)

async function extractAndSync(): Promise<boolean> {
  if (isExtracting) {
    log.debug('Already extracting, skipping')
    return false
  }

  const rootElement = document.getElementById('mylibrary')
  if (!shouldExtract(window.location.hostname, rootElement)) {
    log.debug('Skip extraction: target container not found')
    return false
  }

  isExtracting = true

  try {
    const games = extractAllGames()
    if (games.length === 0) {
      log.info('No games found yet')
      return false
    }
    // 1) omit 情報を1回で取得（packSet と parentMap を同時に）
    const { packSet, parentMap } = await fetchOmitWorks()
    // 2) パック要素はfetchで詳細取得し、配列で受け取る
    const packOnly = games.filter(g => packSet.has(g.storeId))
    const normalGames = games.filter(g => !packSet.has(g.storeId))
    const packGames: DmmExtractedGame[] = packOnly.length > 0
      ? await processPacks(new Set(packOnly.map(g => g.storeId)), parentMap)
      : []

    // 3) パック配下ゲームと通常ゲームを結合し、一度だけ同期
    const allGames: DmmExtractedGame[] = [...normalGames, ...packGames]
    await syncDmmGames(allGames)
    return true
  }
  catch (error) {
    log.error('Extraction failed:', error)
    showInPageNotification('DMM: ゲーム情報の抽出に失敗しました', 'error')
    return false
  }
  finally {
    isExtracting = false
  }
}

function shouldContinuePolling(url: string): boolean {
  const startedAt = extractionStartedAtByUrl.get(url) ?? Date.now()
  extractionStartedAtByUrl.set(url, startedAt)
  return Date.now() - startedAt < EXTRACTION_RETRY_WINDOW_MS
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

  await waitForPageLoad(500)

  if (url !== window.location.href || lastSyncedUrls.has(url)) {
    return
  }

  log.info('Target page detected - Extracting on DMM')
  const synced = await extractAndSync()
  if (synced) {
    lastSyncedUrls.add(url)
    extractionStartedAtByUrl.delete(url)
    return
  }

  if (shouldContinuePolling(url)) {
    scheduleExtraction(2_000)
    return
  }

  log.info('No games found before retry window elapsed')
}

function scheduleExtraction(delay = 500): void {
  if (lastSyncedUrls.has(window.location.href)) {
    return
  }

  if (extractionTimer) {
    clearTimeout(extractionTimer)
  }

  extractionTimer = setTimeout(() => {
    extractionTimer = null
    void initDmmExtractor()
  }, delay)
}

function setupPageChangeObserver(): void {
  const observer = new MutationObserver(() => {
    if (window.location.href !== currentUrl) {
      currentUrl = window.location.href
      extractionStartedAtByUrl.delete(currentUrl)
      scheduleExtraction(0)
      void initLaunchergDownloadOnceForUrl(currentUrl, mark, isMarked)
      return
    }

    scheduleExtraction(500)
  })
  observer.observe(document.body, { childList: true, subtree: true })
}

function main(): void {
  log.info('Script loaded')
  addNotificationStyles()
  setupPageChangeObserver()
  setTimeout(() => {
    void initDmmExtractor()
    void initLaunchergDownloadOnceForUrl(window.location.href, mark, isMarked)
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
