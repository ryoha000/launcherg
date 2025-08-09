import { create, toJson } from '@bufbuild/protobuf'
import {
  addNotificationStyles,
  logger,
  sendExtensionRequest,
  setLogLevel,
  showNotification,
  waitForPageLoad,
} from '@launcherg/shared'
import { DmmGameSchema, DmmSyncGamesRequestSchema, ExtensionRequestSchema } from '@launcherg/shared/proto/extension_internal'

import { processGames } from './data-processor'
import { extractAllGames, shouldExtract } from './dom-extractor'

let isExtracting = false
let currentUrl = window.location.href
const log = logger('dmm-extractor')
let pollingTimerId: number | null = null
let didInitialWait = false

async function extractAndSync(): Promise<void> {
  if (isExtracting) {
    log.debug('Already extracting, skipping')
    return
  }

  isExtracting = true

  try {
    if (!didInitialWait) {
      await waitForPageLoad(2000)
      didInitialWait = true
    }

    const games = extractAllGames()
    if (games.length === 0) {
      log.info('No games found')
      return
    }

    log.info(`Found ${games.length} games`)

    const processedGames = processGames(games)

    const dmmGames = processedGames.map(g => create(DmmGameSchema, {
      id: g.store_id || '',
      category: g.additional_data?.category || '',
      subcategory: g.additional_data?.subcategory || '',
    }))

    const request = create(ExtensionRequestSchema, {
      requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
      request: {
        case: 'syncDmmGames',
        value: create(DmmSyncGamesRequestSchema, {
          games: dmmGames,
          source: 'dmm-extractor',
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
    showNotification('DMM: ゲーム情報の抽出に失敗しました', 'error')
  }
  finally {
    isExtracting = false
  }
}

function initDmmExtractor(): void {
  const rootElement = document.getElementById('mylibrary')
  if (shouldExtract(window.location.hostname, rootElement)) {
    log.info('Target page detected - Starting extraction on DMM')
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

function setupPageChangeObserver(): void {
  const observer = new MutationObserver(() => {
    if (window.location.href !== currentUrl) {
      currentUrl = window.location.href
      setTimeout(() => {
        didInitialWait = false
        initDmmExtractor()
      }, 2000)
    }
  })
  observer.observe(document.body, { childList: true, subtree: true })
}

function main(): void {
  log.info('Script loaded')
  addNotificationStyles()
  setupPageChangeObserver()
  setTimeout(() => {
    initDmmExtractor()
  }, 1000)
}

if ((import.meta as any).env?.MODE === 'development') {
  setLogLevel('debug')
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
