import type { Browser, HandlerContext } from './shared/types'
import { logger } from '@launcherg/shared'
import { createBrowser } from './adapter/browser'
import { createEgsResolver } from './adapter/egs/resolver'
import { createNativeMessenger } from './adapter/native/send'
import { createSyncPool } from './adapter/pool'
import { createMessageDispatcher } from './inbound/dispatcher'
import { performPeriodicSync } from './usecase/periodic'
import { SYNC_GAME_ALARM, syncGame } from './usecase/syncGameScheduler'

const log = logger('background')

const nativeHostName = 'moe.ryoha.launcherg.extension_host'

function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
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
  syncPool: createSyncPool(),
}

const handle = createMessageDispatcher(context)

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
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

chrome.alarms.create(SYNC_GAME_ALARM, {
  periodInMinutes: 1,
})

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'periodic_sync') {
    void performPeriodicSync(browser)
    return
  }
  if (alarm.name === SYNC_GAME_ALARM) {
    void syncGame(context)
  }
})

log.info('Service worker initialized')

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
