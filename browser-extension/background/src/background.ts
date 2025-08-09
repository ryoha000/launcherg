// ブラウザ拡張機能のバックグラウンドスクリプト
// Content ScriptとNative Messaging Hostの橋渡しを行う

// Extension Internal and Native Messaging types
import type {
  DebugNativeMessageRequest,
  DlsiteSyncGamesRequest,
  DmmSyncGamesRequest,
  GetStatusRequest,
  ShowNotificationRequest,
} from '@launcherg/shared/proto/extension_internal'

import type {
  NativeMessage,
  NativeResponse,
} from '@launcherg/shared/proto/native_messaging'

import { create, fromJson, toJson, toJsonString } from '@bufbuild/protobuf'

import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import { logger } from '@launcherg/shared'

import {
  DebugNativeMessageResponseSchema,
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GetStatusResponseSchema,
  ShowNotificationResponseSchema,
  StatusDataSchema,
  SyncGamesResponseSchema,
  SyncResultSchema,
} from '@launcherg/shared/proto/extension_internal'
import {
  HealthCheckRequestSchema,
  DlsiteSyncGamesRequestSchema as NativeDlsiteSyncGamesRequestSchema,
  DmmSyncGamesRequestSchema as NativeDmmSyncGamesRequestSchema,
  GetStatusRequestSchema as NativeGetStatusRequestSchema,
  NativeMessageSchema,
  NativeResponseSchema,
} from '@launcherg/shared/proto/native_messaging'

// 型定義はprotobufから取得するため、interfaceは削除
const log = logger('background')

class BackgroundService {
  private nativeHostName = 'moe.ryoha.launcherg.extension_host'
  private static readonly AGGREGATE_ALARM = 'notify_aggregate'
  private static readonly AGGREGATE_COUNT_KEY = 'aggregate_sync_count'

  constructor() {
    this.setupMessageListeners()
    this.setupAlarms()
  }

  private setupMessageListeners(): void {
    // Content Scriptからのメッセージを処理
    chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
      this.handleMessage(message, _sender, sendResponse)
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
        return
      }
      if (alarm.name === BackgroundService.AGGREGATE_ALARM) {
        void this.fireAggregateNotification()
      }
    })
  }

  private async handleMessage(
    message: any,
    _sender: chrome.runtime.MessageSender,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      // プロトバフメッセージをJSONから復元
      const extensionRequest = fromJson(ExtensionRequestSchema, message)

      // リクエストタイプに応じて処理を分岐
      switch (extensionRequest.request.case) {
        case 'syncDmmGames': {
          await this.handleProtobufSyncDmmGames(
            extensionRequest.requestId,
            extensionRequest.request.value,
            sendResponse,
          )
          break
        }
        case 'syncDlsiteGames': {
          await this.handleProtobufSyncDlsiteGames(
            extensionRequest.requestId,
            extensionRequest.request.value,
            sendResponse,
          )
          break
        }

        case 'getConfig': {
          log.warn('getConfig is deprecated')
          const errorResponse = create(ExtensionResponseSchema, {
            requestId: extensionRequest.requestId,
            success: false,
            error: 'getConfig is deprecated',
            response: { case: undefined },
          })
          sendResponse(toJson(ExtensionResponseSchema, errorResponse))
          break
        }

        case 'showNotification':
          await this.handleProtobufShowNotification(
            extensionRequest.requestId,
            extensionRequest.request.value,
            sendResponse,
          )
          break

        case 'getStatus':
          await this.handleProtobufGetStatus(
            extensionRequest.requestId,
            extensionRequest.request.value,
            sendResponse,
          )
          break

        case 'debugNativeMessage':
          await this.handleProtobufDebugNativeMessage(
            extensionRequest.requestId,
            extensionRequest.request.value,
            sendResponse,
          )
          break

        default: {
          log.warn('Unknown request type:', extensionRequest.request.case)
          const errorResponse = create(ExtensionResponseSchema, {
            requestId: extensionRequest.requestId,
            success: false,
            error: 'Unknown request type',
            response: { case: undefined },
          })
          sendResponse(toJson(ExtensionResponseSchema, errorResponse))
        }
      }
    }
    catch (error) {
      log.error('Error handling message:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      // エラーレスポンスを作成
      const errorResponse = create(ExtensionResponseSchema, {
        requestId: (message as any).requestId || 'unknown',
        success: false,
        error: errorMessage,
        response: { case: undefined },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private async handleProtobufSyncDmmGames(
    requestId: string,
    syncGamesRequest: DmmSyncGamesRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    log.info(`Syncing ${syncGamesRequest.games.length} DMM games`)
    await this.recordSyncAggregation(syncGamesRequest.games.length || 0)

    try {
      const nativeSyncRequest = create(NativeDmmSyncGamesRequestSchema, {
        games: syncGamesRequest.games.map(g => ({ id: g.id, category: g.category, subcategory: g.subcategory })),
        extensionId: chrome.runtime.id,
      })

      const nativeMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, {
          seconds: BigInt(Math.floor(Date.now() / 1000)),
        }),
        requestId: this.generateRequestId(),
        message: {
          case: 'syncDmmGames',
          value: nativeSyncRequest,
        },
      })

      const nativeResponse = await this.sendNativeProtobufMessage(
        nativeMessage,
      )

      if (nativeResponse && nativeResponse.success) {
        let syncResult
        if (nativeResponse.response.case === 'syncGamesResult') {
          const syncBatchResult = nativeResponse.response.value
          syncResult = create(SyncResultSchema, {
            successCount: Number(syncBatchResult.successCount),
            errorCount: Number(syncBatchResult.errorCount),
            errors: syncBatchResult.errors,
            syncedGames: syncBatchResult.syncedGames,
          })

          // 成功件数の通知（0件は通知しない）
          const success = Number(syncBatchResult.successCount || 0)
          if (success > 0) {
            const iconUrl = chrome.runtime.getURL('icons/icon32.png')
            await chrome.notifications.create({
              type: 'basic',
              iconUrl,
              title: 'DMM 同期',
              message: `新規 ${success} 件を同期しました`,
            })
          }
          else {
            // 通知不要、警告ログのみ
            console.warn('DMM: 同期成功 0 件（重複のみの可能性）')
          }
        }

        const response = create(ExtensionResponseSchema, {
          requestId,
          success: true,
          error: '',
          response: {
            case: 'syncGamesResult',
            value: create(SyncGamesResponseSchema, {
              result: syncResult,
              message: `DMMから${syncGamesRequest.games.length}個のゲームを同期しました`,
            }),
          },
        })

        sendResponse(toJson(ExtensionResponseSchema, response))
      }
      else {
        throw new Error(nativeResponse?.error || 'Native host returned error')
      }
    }
    catch (error) {
      log.error('Sync failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      const errorResponse = create(ExtensionResponseSchema, {
        requestId,
        success: false,
        error: errorMessage,
        response: {
          case: 'syncGamesResult',
          value: create(SyncGamesResponseSchema, {
            message: `DMMの同期に失敗しました`,
          }),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private async handleProtobufSyncDlsiteGames(
    requestId: string,
    syncGamesRequest: DlsiteSyncGamesRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    log.info(`Syncing ${syncGamesRequest.games.length} DLsite games`)
    await this.recordSyncAggregation(syncGamesRequest.games.length || 0)

    try {
      const nativeSyncRequest = create(NativeDlsiteSyncGamesRequestSchema, {
        games: syncGamesRequest.games.map(g => ({ id: g.id, category: g.category })),
        extensionId: chrome.runtime.id,
      })

      const nativeMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, {
          seconds: BigInt(Math.floor(Date.now() / 1000)),
        }),
        requestId: this.generateRequestId(),
        message: {
          case: 'syncDlsiteGames',
          value: nativeSyncRequest,
        },
      })

      const nativeResponse = await this.sendNativeProtobufMessage(
        nativeMessage,
      )

      if (nativeResponse && nativeResponse.success) {
        let syncResult
        if (nativeResponse.response.case === 'syncGamesResult') {
          const syncBatchResult = nativeResponse.response.value
          syncResult = create(SyncResultSchema, {
            successCount: Number(syncBatchResult.successCount),
            errorCount: Number(syncBatchResult.errorCount),
            errors: syncBatchResult.errors,
            syncedGames: syncBatchResult.syncedGames,
          })

          // 成功件数の通知（0件は通知しない）
          const success = Number(syncBatchResult.successCount || 0)
          if (success > 0) {
            const iconUrl = chrome.runtime.getURL('icons/icon32.png')
            await chrome.notifications.create({
              type: 'basic',
              iconUrl,
              title: 'DLsite 同期',
              message: `新規 ${success} 件を同期しました`,
            })
          }
          else {
            // 通知不要、警告ログのみ
            console.warn('DLsite: 同期成功 0 件（重複のみの可能性）')
          }
        }

        const response = create(ExtensionResponseSchema, {
          requestId,
          success: true,
          error: '',
          response: {
            case: 'syncGamesResult',
            value: create(SyncGamesResponseSchema, {
              result: syncResult,
              message: `DLsiteから${syncGamesRequest.games.length}個のゲームを同期しました`,
            }),
          },
        })

        sendResponse(toJson(ExtensionResponseSchema, response))
      }
      else {
        throw new Error(nativeResponse?.error || 'Native host returned error')
      }
    }
    catch (error) {
      log.error('Sync failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      const errorResponse = create(ExtensionResponseSchema, {
        requestId,
        success: false,
        error: errorMessage,
        response: {
          case: 'syncGamesResult',
          value: create(SyncGamesResponseSchema, {
            message: `DLsiteの同期に失敗しました`,
          }),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private async handleProtobufShowNotification(
    requestId: string,
    notificationRequest: ShowNotificationRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      const iconPath
        = notificationRequest.iconType === 'error'
          ? 'icons/icon32_error.png'
          : 'icons/icon32.png'
      const iconUrl = chrome.runtime.getURL(iconPath)
      await chrome.notifications.create({
        type: 'basic',
        iconUrl,
        title: notificationRequest.title,
        message: notificationRequest.message,
      })

      const response = create(ExtensionResponseSchema, {
        requestId,
        success: true,
        error: '',
        response: {
          case: 'notificationResult',
          value: create(ShowNotificationResponseSchema, {}),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, response))
    }
    catch (error) {
      log.error('Notification failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      const errorResponse = create(ExtensionResponseSchema, {
        requestId,
        success: false,
        error: errorMessage,
        response: { case: undefined },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private async recordSyncAggregation(count: number): Promise<void> {
    const current = await this.getAggregateCount()
    await this.setAggregateCount(current + count)
    // 30秒後に一度だけ発火するアラームを再スケジュール
    chrome.alarms.create(BackgroundService.AGGREGATE_ALARM, {
      when: Date.now() + 30_000,
    })
  }

  private async fireAggregateNotification(): Promise<void> {
    const total = await this.getAggregateCount()
    if (total > 0) {
      const title = 'Launcherg DL Store Sync'
      const message = `過去30秒間に合計${total}件を同期しました`
      await chrome.notifications.create({
        type: 'basic',
        iconUrl: chrome.runtime.getURL('icons/icon32.png'),
        title,
        message,
      })
      await this.setAggregateCount(0)
    }
  }

  private async getAggregateCount(): Promise<number> {
    return new Promise((resolve) => {
      chrome.storage.local.get(
        [BackgroundService.AGGREGATE_COUNT_KEY],
        (items) => {
          const value = items[BackgroundService.AGGREGATE_COUNT_KEY]
          resolve(typeof value === 'number' ? value : 0)
        },
      )
    })
  }

  private async setAggregateCount(value: number): Promise<void> {
    return new Promise((resolve) => {
      chrome.storage.local.set(
        { [BackgroundService.AGGREGATE_COUNT_KEY]: value },
        () => resolve(),
      )
    })
  }

  private async handleProtobufGetStatus(
    requestId: string,
    _getStatusRequest: GetStatusRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      const nativeMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, {
          seconds: BigInt(Math.floor(Date.now() / 1000)),
        }),
        requestId: this.generateRequestId(),
        message: {
          case: 'getStatus',
          value: create(NativeGetStatusRequestSchema, {}),
        },
      })

      const nativeResponse = await this.sendNativeProtobufMessage(
        nativeMessage,
      )

      let statusData
      if (nativeResponse && nativeResponse.response.case === 'statusResult') {
        const syncStatus = nativeResponse.response.value
        statusData = create(StatusDataSchema, {
          lastSync: syncStatus.lastSync
            ? new Date(Number(syncStatus.lastSync.seconds) * 1000).toISOString()
            : '',
          totalSynced: Number(syncStatus.totalSynced),
          connectedExtensions: syncStatus.connectedExtensions,
          isRunning: syncStatus.isRunning,
          connectionStatus: syncStatus.connectionStatus.toString(),
          errorMessage: syncStatus.errorMessage,
        })
      }

      const response = create(ExtensionResponseSchema, {
        requestId,
        success: true,
        error: '',
        response: {
          case: 'statusResult',
          value: create(GetStatusResponseSchema, {
            status: statusData,
          }),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, response))
    }
    catch (error) {
      log.error('Get status failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      const errorResponse = create(ExtensionResponseSchema, {
        requestId,
        success: false,
        error: errorMessage,
        response: { case: undefined },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private async sendNativeProtobufMessage(
    message: NativeMessage,
  ): Promise<NativeResponse | null> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Native messaging timeout'))
      }, 30000)

      // JSONとして送信（ProtoBuf専用のシリアライザを使用）
      const jsonString = toJsonString(NativeMessageSchema, message)
      log.debug('Sending native message:', jsonString, toJson(NativeMessageSchema, message))

      chrome.runtime.sendNativeMessage(
        this.nativeHostName,
        // @ts-expect-error nullになりえるらしいがいったん無視
        toJson(NativeMessageSchema, message),
        (response) => {
          clearTimeout(timeout)

          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message))
          }
          else if (response) {
            try {
              // JSONレスポンスをProtoBuf形式にパース
              // const jsonString = JSON.stringify(response)
              const nativeResponse = fromJson(NativeResponseSchema, response)
              log.debug('Received native response:', nativeResponse)
              try {
                const responseJson = toJson(NativeResponseSchema, nativeResponse)
                log.info('Native host response (json):', JSON.stringify(responseJson))
              }
              catch (jsonErr) {
                log.warn('Failed to serialize native response to JSON:', jsonErr)
              }
              resolve(nativeResponse)
            }
            catch (e) {
              log.error('Failed to parse JSON response:', e)
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
    log.info('Performing periodic sync check')

    // アクティブなタブでDMM/DLsiteのページがあるかチェック
    const tabs = await chrome.tabs.query({
      url: ['https://games.dmm.co.jp/*', 'https://play.dlsite.com/*'],
    })

    for (const tab of tabs) {
      if (tab.id) {
        // タブにメッセージを送信して同期をトリガー
        chrome.tabs.sendMessage(tab.id, { type: 'periodic_sync_check' })
      }
    }
  }

  private async handleProtobufDebugNativeMessage(
    requestId: string,
    _debugRequest: DebugNativeMessageRequest,
    sendResponse: (response?: any) => void,
  ): Promise<void> {
    try {
      const debugMessage = create(NativeMessageSchema, {
        timestamp: create(TimestampSchema, {
          seconds: BigInt(Math.floor(Date.now() / 1000)),
        }),
        requestId: this.generateRequestId(),
        message: {
          case: 'healthCheck',
          value: create(HealthCheckRequestSchema, {}),
        },
      })

      const nativeResponse = await this.sendNativeProtobufMessage(debugMessage)

      const response = create(ExtensionResponseSchema, {
        requestId,
        success: true,
        error: '',
        response: {
          case: 'debugResult',
          value: create(DebugNativeMessageResponseSchema, {
            nativeResponseJson: JSON.stringify(nativeResponse),
            timestamp: new Date().toISOString(),
          }),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, response))
    }
    catch (error) {
      log.error('Debug native message failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'

      const errorResponse = create(ExtensionResponseSchema, {
        requestId,
        success: false,
        error: errorMessage,
        response: {
          case: 'debugResult',
          value: create(DebugNativeMessageResponseSchema, {
            nativeResponseJson: '',
            timestamp: new Date().toISOString(),
          }),
        },
      })
      sendResponse(toJson(ExtensionResponseSchema, errorResponse))
    }
  }

  private generateRequestId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2)
  }
}

// Service Worker起動時に初期化
const backgroundService = new BackgroundService()
log.info('Service worker initialized')

// 拡張機能インストール時の処理
chrome.runtime.onInstalled.addListener((details) => {
  log.info('Extension installed:', details.reason)

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
chrome.tabs.onUpdated.addListener((_tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    const isDMMGames = tab.url.includes('games.dmm.co.jp')
    const isDLsite = tab.url.includes('dlsite.com')

    if (isDMMGames || isDLsite) {
      log.debug(`Target site loaded: ${tab.url}`)
      // 必要に応じて自動同期をトリガー
    }
  }
})

export default backgroundService
