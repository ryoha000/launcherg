// ブラウザ拡張機能のバックグラウンドスクリプト
// Content ScriptとNative Messaging Hostの橋渡しを行う

import type { SiteConfig } from '../content-scripts/base-extractor'
import extractionRules from '../config/extraction-rules.json'

interface SyncRequest {
  type: 'sync_games'
  store: 'DMM' | 'DLSite'
  games: any[]
  source: string
}

interface ConfigRequest {
  type: 'get_config'
  site: 'dmm' | 'dlsite'
}

interface NotificationRequest {
  type: 'show_notification'
  title: string
  message: string
  iconType?: 'success' | 'error'
}

class BackgroundService {
  private nativeHostName = 'moe.ryoha.launcherg.extension_host'
  private configs: Record<string, SiteConfig> = {}

  constructor() {
    this.initializeConfigs()
    this.setupMessageListeners()
    this.setupAlarms()
  }

  private initializeConfigs(): void {
    // 設定を読み込み
    this.configs = {
      dmm: extractionRules.sites.dmm as unknown as SiteConfig,
      dlsite: extractionRules.sites.dlsite as unknown as SiteConfig,
    }

    console.log('[Background] Configs loaded:', Object.keys(this.configs))
  }

  private setupMessageListeners(): void {
    // Content Scriptからのメッセージを処理
    chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender, sendResponse)
      return true // 非同期レスポンスを有効にする
    })
  }

  private setupAlarms(): void {
    // 定期的な同期のためのアラーム設定
    chrome.alarms.create('periodic_sync', {
      delayInMinutes: 5,
      periodInMinutes: 30,
    })

    chrome.alarms.onAlarm.addListener((alarm) => {
      if (alarm.name === 'periodic_sync') {
        this.performPeriodicSync()
      }
    })
  }

  private async handleMessage(
    message: any,
    sender: chrome.runtime.MessageSender,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      switch (message.type) {
        case 'sync_games':
          await this.handleSyncGames(message as SyncRequest, sendResponse)
          break

        case 'get_config':
          this.handleGetConfig(message as ConfigRequest, sendResponse)
          break

        case 'show_notification':
          await this.handleShowNotification(message as NotificationRequest, sendResponse)
          break

        case 'get_status':
          await this.handleGetStatus(sendResponse)
          break

        case 'debug_native_message':
          await this.handleDebugNativeMessage(message, sendResponse)
          break

        default:
          console.warn('[Background] Unknown message type:', message.type)
          sendResponse({ success: false, error: 'Unknown message type' })
      }
    }
    catch (error) {
      console.error('[Background] Error handling message:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      sendResponse({ success: false, error: errorMessage })
    }
  }

  private async handleSyncGames(
    request: SyncRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    console.log(`[Background] Syncing ${request.games.length} games from ${request.store}`)

    try {
      // Native Messaging Hostに送信するメッセージを準備
      const nativeMessage = {
        type: 'sync_games',
        payload: {
          store: request.store,
          games: request.games,
          extension_id: chrome.runtime.id,
        },
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      }

      // Native Messaging Hostに送信
      const response = await this.sendNativeMessage(nativeMessage)

      if (response && response.success) {
        console.log('[Background] Native host sync successful:', response)
        sendResponse({
          success: true,
          result: response.data,
          message: `${request.store}から${request.games.length}個のゲームを同期しました`,
        })
      }
      else {
        throw new Error(response?.error || 'Native host returned error')
      }
    }
    catch (error) {
      console.error('[Background] Sync failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      sendResponse({
        success: false,
        error: errorMessage,
        message: `${request.store}の同期に失敗しました`,
      })
    }
  }

  private handleGetConfig(
    request: ConfigRequest,
    sendResponse: (response?: any) => void,
  ): void {
    const config = this.configs[request.site]
    if (config) {
      sendResponse({ success: true, config })
    }
    else {
      sendResponse({ success: false, error: `Config not found for site: ${request.site}` })
    }
  }

  private async handleShowNotification(
    request: NotificationRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      await chrome.notifications.create({
        type: 'basic',
        iconUrl: request.iconType === 'error' ? 'icons/icon32_error.png' : 'icons/icon32.png',
        title: request.title,
        message: request.message,
      })
      sendResponse({ success: true })
    }
    catch (error) {
      console.error('[Background] Notification failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      sendResponse({ success: false, error: errorMessage })
    }
  }

  private async handleGetStatus(sendResponse: (response?: any) => void): Promise<void> {
    try {
      const nativeMessage = {
        type: 'get_status',
        payload: {},
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      }

      const response = await this.sendNativeMessage(nativeMessage)
      sendResponse({ success: true, status: response?.data })
    }
    catch (error) {
      console.error('[Background] Get status failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      sendResponse({ success: false, error: errorMessage })
    }
  }

  private async sendNativeMessage(message: any): Promise<any> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Native messaging timeout'))
      }, 30000) // 30秒タイムアウト

      chrome.runtime.sendNativeMessage(
        this.nativeHostName,
        message,
        (response) => {
          clearTimeout(timeout)

          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message))
          }
          else {
            resolve(response)
          }
        },
      )
    })
  }

  private async performPeriodicSync(): Promise<void> {
    console.log('[Background] Performing periodic sync check')

    // アクティブなタブでDMM/DLsiteのページがあるかチェック
    const tabs = await chrome.tabs.query({
      url: ['https://games.dmm.co.jp/*', 'https://www.dlsite.com/*'],
    })

    for (const tab of tabs) {
      if (tab.id) {
        // タブにメッセージを送信して同期をトリガー
        chrome.tabs.sendMessage(tab.id, { type: 'periodic_sync_check' })
      }
    }
  }

  private async handleDebugNativeMessage(
    message: any,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    console.log('[Background] Debug native message request:', message)

    try {
      if (!message.payload) {
        throw new Error('Debug message payload is missing')
      }

      // デバッグメッセージの検証
      const debugMessage = message.payload
      if (!debugMessage.type) {
        throw new Error('Debug message must have a type field')
      }

      // request_idとtimestampが未設定の場合は自動生成
      if (!debugMessage.request_id) {
        debugMessage.request_id = this.generateRequestId()
      }
      if (!debugMessage.timestamp) {
        debugMessage.timestamp = new Date().toISOString()
      }

      console.log('[Background] Sending debug native message:', debugMessage)

      // Native Messaging Hostに直接送信
      const response = await this.sendNativeMessage(debugMessage)

      console.log('[Background] Debug native message response:', response)

      // レスポンスをそのまま返す（成功・失敗問わず）
      sendResponse({
        success: true,
        debug_request: debugMessage,
        native_response: response,
        timestamp: new Date().toISOString(),
      })
    }
    catch (error) {
      console.error('[Background] Debug native message failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      
      sendResponse({
        success: false,
        error: errorMessage,
        debug_request: message.payload,
        timestamp: new Date().toISOString(),
      })
    }
  }

  private generateRequestId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2)
  }
}

// Service Worker起動時に初期化
const backgroundService = new BackgroundService()

console.log('[Background] Service worker initialized')

// 拡張機能インストール時の処理
chrome.runtime.onInstalled.addListener((details) => {
  console.log('[Background] Extension installed:', details.reason)

  if (details.reason === 'install') {
    // 初回インストール時の処理
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

// タブ更新時の処理
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    const isDMMGames = tab.url.includes('games.dmm.co.jp')
    const isDLsite = tab.url.includes('dlsite.com')

    if (isDMMGames || isDLsite) {
      console.log(`[Background] Target site loaded: ${tab.url}`)
      // 必要に応じて自動同期をトリガー
    }
  }
})

export default backgroundService
