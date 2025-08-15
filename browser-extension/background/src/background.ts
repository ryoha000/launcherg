// ブラウザ拡張機能のバックグラウンドスクリプト（薄い初期化）
import { logger } from '@launcherg/shared'
import { AGGREGATE_ALARM, fireAggregateNotification, recordSyncAggregation } from './aggregation'
import { resolveEgsForDlsite, resolveEgsForDmm } from './egs-resolver'
import { createMessageHandler } from './handler/handler'
import { sendNativeProtobufMessage } from './native/send'
import { performPeriodicSync } from './periodic'

const log = logger('background')

const nativeHostName = 'moe.ryoha.launcherg.extension_host'

function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

const handle = createMessageHandler({
  nativeHostName,
  extensionId: chrome.runtime.id,
  sendNativeProtobufMessage,
  generateRequestId,
  resolveEgsForDmm,
  resolveEgsForDlsite,
  recordSyncAggregation,
})

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

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'periodic_sync') {
    void performPeriodicSync()
    return
  }
  if (alarm.name === AGGREGATE_ALARM) {
    void fireAggregateNotification()
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
