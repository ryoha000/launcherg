// ブラウザ拡張機能のバックグラウンドスクリプト
// Content ScriptとNative Messaging Hostの橋渡しを行う

import type { SiteConfig } from '../content-scripts/base-extractor'
// Extension Internal types

// Native Messaging types
import type {
  NativeMessage,
  NativeResponse,
} from '../proto/native_messaging/common_pb'

import type {
  JsonConfigResponse,
  JsonDebugNativeMessageMessage,
  JsonDebugResponse,
  JsonGetConfigMessage,
  JsonMessage,
  JsonResponse,
  JsonShowNotificationMessage,
  JsonStatusResponse,
  JsonSyncGamesMessage,
  JsonSyncResponse,
} from '../types/json-messages'

import { create, fromBinary, fromJsonString, toBinary, toJsonString } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import extractionRules from '../config/extraction-rules.json'

import {
  HealthCheckRequestSchema,
  GetStatusRequestSchema as NativeGetStatusRequestSchema,
  NativeMessageSchema,
  NativeResponseSchema,
} from '../proto/native_messaging/common_pb'
import {
  ExtractedGameDataSchema,
  SyncGamesRequestSchema as NativeSyncGamesRequestSchema,
} from '../proto/native_messaging/sync_pb'

// 型定義はprotobufから取得するため、interfaceは削除

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
    message: JsonMessage,
    sender: chrome.runtime.MessageSender,
    sendResponse: (response?: JsonResponse) => void,
  ): Promise<void> {
    try {
      // 従来のJSON形式メッセージを処理
      switch (message.type) {
        case 'sync_games':
          await this.handleSyncGames(message as JsonSyncGamesMessage, sendResponse)
          break

        case 'get_config':
          this.handleGetConfig(message as JsonGetConfigMessage, sendResponse)
          break

        case 'show_notification':
          await this.handleShowNotification(message as JsonShowNotificationMessage, sendResponse)
          break

        case 'get_status':
          await this.handleGetStatus(sendResponse)
          break

        case 'debug_native_message':
          await this.handleDebugNativeMessage(message as JsonDebugNativeMessageMessage, sendResponse)
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
    request: JsonSyncGamesMessage,
    sendResponse: (response?: JsonSyncResponse) => void,
  ): Promise<void> {
    console.log(`[Background] Syncing ${request.games.length} games from ${request.store}`)

    try {
      const extractedGames = request.games.map(game => create(ExtractedGameDataSchema, {
        storeId: game.store_id || '',
        title: game.title || '',
        purchaseUrl: game.purchase_url || '',
        purchaseDate: game.purchase_date || '',
        thumbnailUrl: game.thumbnail_url || '',
        additionalData: game.additional_data || {},
      }))

      const syncRequest = create(NativeSyncGamesRequestSchema, {
        store: request.store,
        games: extractedGames,
        extensionId: chrome.runtime.id,
      })

      const nativeMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, { seconds: BigInt(Math.floor(Date.now() / 1000)) }),
        requestId: this.generateRequestId(),
        message: {
          case: 'syncGames',
          value: syncRequest,
        },
      })

      const response = await this.sendNativeProtobufMessage(nativeMessage)

      if (response && response.success) {
        let resultData: JsonSyncResponse['result']
        if (response.response.case === 'syncGamesResult') {
          const syncBatchResult = response.response.value
          resultData = {
            successCount: Number(syncBatchResult.successCount),
            errorCount: Number(syncBatchResult.errorCount),
            errors: syncBatchResult.errors,
            syncedGames: syncBatchResult.syncedGames,
          }
        }

        sendResponse({
          success: true,
          result: resultData,
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
    request: JsonGetConfigMessage,
    sendResponse: (response?: JsonConfigResponse) => void,
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
    request: JsonShowNotificationMessage,
    sendResponse: (response?: JsonResponse) => void,
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

  private async handleGetStatus(sendResponse: (response?: JsonStatusResponse) => void): Promise<void> {
    try {
      const nativeMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, { seconds: BigInt(Math.floor(Date.now() / 1000)) }),
        requestId: this.generateRequestId(),
        message: {
          case: 'getStatus',
          value: create(NativeGetStatusRequestSchema, {}),
        },
      })

      const response = await this.sendNativeProtobufMessage(nativeMessage)

      let statusData: JsonStatusResponse['status']
      if (response && response.response.case === 'statusResult') {
        const syncStatus = response.response.value
        statusData = {
          lastSync: syncStatus.lastSync ? new Date(Number(syncStatus.lastSync.seconds) * 1000).toISOString() : '',
          totalSynced: Number(syncStatus.totalSynced),
          connectedExtensions: syncStatus.connectedExtensions,
          isRunning: syncStatus.isRunning,
          connectionStatus: syncStatus.connectionStatus.toString(),
          errorMessage: syncStatus.errorMessage,
        }
      }

      sendResponse({ success: true, status: statusData })
    }
    catch (error) {
      console.error('[Background] Get status failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      sendResponse({ success: false, error: errorMessage })
    }
  }

  private async sendNativeProtobufMessage(message: NativeMessage): Promise<NativeResponse | null> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Native messaging timeout'))
      }, 30000)

      // JSONとして送信（ProtoBuf専用のシリアライザを使用）
      const jsonString = toJsonString(NativeMessageSchema, message)

      chrome.runtime.sendNativeMessage(
        this.nativeHostName,
        JSON.parse(jsonString),
        (response) => {
          clearTimeout(timeout)

          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message))
          }
          else if (response) {
            try {
              // JSONレスポンスをProtoBuf形式にパース
              const jsonString = JSON.stringify(response)
              const nativeResponse = fromJsonString(NativeResponseSchema, jsonString)
              resolve(nativeResponse)
            }
            catch (e) {
              console.error('[Background] Failed to parse JSON response:', e)
              reject(e)
            }
          }
          else {
            resolve(null)
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
    message: JsonDebugNativeMessageMessage,
    sendResponse: (response?: JsonDebugResponse) => void,
  ): Promise<void> {
    try {
      const debugMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, { seconds: BigInt(Math.floor(Date.now() / 1000)) }),
        requestId: this.generateRequestId(),
        message: {
          case: 'healthCheck',
          value: create(HealthCheckRequestSchema, {}),
        },
      })

      const response = await this.sendNativeProtobufMessage(debugMessage)

      sendResponse({
        success: true,
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
