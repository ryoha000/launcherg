import type { Browser, HandlerContext, SyncCoordinator } from './shared/types'
import { discardFailedDownloadIntent, logger, registerDownloadIntent } from '@launcherg/shared'
import { createBrowser } from './adapter/browser'
import { createEgsResolver } from './adapter/egs/resolver'
import { createNativeMessenger } from './adapter/native/send'
import { createMessageDispatcher } from './inbound/dispatcher'
import { setupDownloadsHandler, startIntentDownloads } from './usecase/downloads'
import { performPeriodicSync } from './usecase/periodic'

const log = logger('background')

const nativeHostName = 'moe.ryoha.launcherg.extension_host'

function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

function createSyncCoordinator(): SyncCoordinator {
  let tail = Promise.resolve<void>(undefined)

  return {
    async runExclusive<T>(callback: () => Promise<T>): Promise<T> {
      const current = tail.catch(() => undefined).then(callback)
      tail = current.then(() => undefined, () => undefined)
      return await current
    },
  }
}

const nativeMessenger = createNativeMessenger(nativeHostName)
const egsResolver = createEgsResolver()
const browser: Browser = createBrowser()

const context: HandlerContext = {
  extensionId: chrome.runtime.id,
  nativeHostName,
  nativeMessenger,
  egsResolver,
  idGenerator: { generate: generateRequestId },
  browser,
  syncCoordinator: createSyncCoordinator(),
}

const handle = createMessageDispatcher(context)
const retryDownloads = setupDownloadsHandler(context)

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message?.type === 'discard_failed_download') {
    void discardFailedDownloadIntent(message.storeId).then(
      () => sendResponse({ success: true }),
      error => sendResponse({ success: false, error: String(error) }),
    )
    return true
  }
  if (message?.type === 'register_download_intent') {
    void registerDownloadIntent(message.storeId, message.entry).then(
      () => sendResponse({ success: true }),
      error => sendResponse({ success: false, error: String(error) }),
    )
    return true
  }
  if (message?.type === 'retry_downloads') {
    void retryDownloads(true).then(
      () => sendResponse({ success: true }),
      error => sendResponse({ success: false, error: String(error) }),
    )
    return true
  }
  // Lightweight control message from content scripts
  if (message && typeof message === 'object' && (message as any).type === 'close_current_tab') {
    const tabId = sender?.tab?.id
    if (typeof tabId === 'number') {
      chrome.tabs.remove(tabId, () => {
        if (chrome.runtime.lastError)
          sendResponse({ success: false, error: chrome.runtime.lastError.message })
        else
          sendResponse({ success: true })
      })
      return true
    }
    sendResponse({ success: false, error: 'No tab id' })
    return false
  }

  if (message && typeof message === 'object' && (message as any).type === 'start_dmm_downloads') {
    const urls = Array.isArray((message as any).payload?.urls)
      ? (message as any).payload.urls.filter((value: unknown): value is string => typeof value === 'string' && value.length > 0)
      : []

    void startIntentDownloads(message.payload.storeId, urls).then(
      startedCount => sendResponse({ success: true, startedCount, failedUrls: [] }),
      error => sendResponse({ success: false, error: String(error) }),
    )

    return true
  }

  void (async () => {
    const response = await handle(message)
    sendResponse(response)
  })()
  return true
})

chrome.alarms.create('periodic_sync', {
  delayInMinutes: 5,
  periodInMinutes: 30,
})

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'periodic_sync') {
    void performPeriodicSync(browser)
  }
})

log.info('Service worker initialized')

// downloads 完了検知のセットアップ

chrome.runtime.onInstalled.addListener((details) => {
  log.info('Extension installed:', details.reason)
  if (details.reason === 'install') {
    chrome.storage.local.set({
      extension_config: {
        auto_sync: true,
        show_notifications: true,
        debug_mode: false,
        sync_interval: 30,
      },
    })
  }
})

chrome.tabs.onUpdated.addListener((_tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    const isDMMGames = tab.url.includes('dlsoft.dmm.co.jp')
    const isDLsite = tab.url.includes('dlsite.com')
    if (isDMMGames || isDLsite) {
      log.debug(`Target site loaded: ${tab.url}`)
    }
  }
})

export {}
