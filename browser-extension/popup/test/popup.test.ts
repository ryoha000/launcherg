import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { create, fromJson, toJson } from '@bufbuild/protobuf'

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GetStatusResponseSchema,
  StatusDataSchema,
} from '../../shared/src/proto/extension_internal/messages_pb'
// PopupControllerをインポート
import { PopupController } from '../src/popup'

// __dirname の代替
const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// HTMLを読み込む、スクリプトタグを除外
const popupHtmlRaw = readFileSync(resolve(__dirname, '../src/popup.html'), 'utf-8')
const popupHtml = popupHtmlRaw
  .replace(/<script[^>]*>.*?<\/script>/gs, '')
  .replace(/<link[^>]*rel="stylesheet"[^>]*>/g, '')

// テスト環境を設定
process.env.NODE_ENV = 'test'

describe('popupController', () => {
  let _controller: PopupController
  let mockChrome: any

  beforeEach(() => {
    // DOM設定
    document.body.innerHTML = popupHtml

    // Chrome APIモック
    mockChrome = {
      storage: {
        local: {
          get: vi.fn().mockResolvedValue({
            extension_config: {
              auto_sync: true,
              show_notifications: true,
              debug_mode: false,
              sync_interval: 30,
            },
            sync_logs: [],
          }),
          set: vi.fn().mockResolvedValue(undefined),
        },
      },
      runtime: {
        id: 'test-extension-id',
        lastError: null,
        sendMessage: vi.fn().mockImplementation((messageJson, callback) => {
          if (callback) {
            try {
              // JSONからExtensionRequestをデコード
              const request = fromJson(ExtensionRequestSchema, messageJson)

              // GetStatusRequestの場合のレスポンスを作成
              if (request.request.case === 'getStatus') {
                const statusData = create(StatusDataSchema, {
                  lastSync: new Date().toISOString(),
                  totalSynced: 42,
                  connectedExtensions: [],
                  isRunning: true,
                  connectionStatus: 'connected',
                  errorMessage: '',
                })

                const statusResponse = create(GetStatusResponseSchema, {
                  status: statusData,
                })

                const response = create(ExtensionResponseSchema, {
                  requestId: request.requestId,
                  success: true,
                  error: '',
                  response: {
                    case: 'statusResult',
                    value: statusResponse,
                  },
                })

                // JSONとしてシリアライズして返す
                const responseJson = toJson(ExtensionResponseSchema, response)
                callback(responseJson)
              }
              else {
                // その他のリクエストの場合
                const response = create(ExtensionResponseSchema, {
                  requestId: request.requestId,
                  success: true,
                  error: '',
                })
                const responseJson = toJson(ExtensionResponseSchema, response)
                callback(responseJson)
              }
            }
            catch (error) {
              callback({ success: false, error: error.message })
            }
          }
          return true
        }),
      },
      tabs: {
        query: vi.fn().mockResolvedValue([{
          id: 1,
          url: 'https://games.dmm.co.jp/test',
          title: 'Test Page',
        }]),
        sendMessage: vi.fn().mockImplementation((tabId, message, callback) => {
          if (callback) {
            callback({ success: true, message: '5件のゲームを同期しました' })
          }
          return true
        }),
        create: vi.fn(),
      },
    }

    // グローバルに設定
    ;(globalThis as any).chrome = mockChrome
    ;(window as any).chrome = mockChrome

    // navigator.clipboard モック
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
      writable: true,
    })
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('初期化', () => {
    it('popupControllerが正しく初期化される', async () => {
      _controller = new PopupController()

      // 少し待機してPromiseを解決させる
      await new Promise(resolve => setTimeout(resolve, 100))

      // 接続ステータスが更新されていることを確認
      const connectionStatus = document.getElementById('connection-status')
      expect(connectionStatus?.textContent).toBe('接続済み')
      expect(connectionStatus?.className).toContain('connected')

      // ストレージから設定が読み込まれていることを確認
      expect(mockChrome.storage.local.get).toHaveBeenCalledWith(['extension_config', 'sync_logs'])
    })
  })

  describe('設定管理', () => {
    beforeEach(async () => {
      _controller = new PopupController()
      await new Promise(resolve => setTimeout(resolve, 100))
    })

    it('自動同期チェックボックスの変更が保存される', async () => {
      const autoSyncCheckbox = document.getElementById('auto-sync-checkbox') as HTMLInputElement
      expect(autoSyncCheckbox).toBeTruthy()
      expect(autoSyncCheckbox.checked).toBe(true)

      // チェックを外す
      autoSyncCheckbox.checked = false
      autoSyncCheckbox.dispatchEvent(new Event('change'))

      await new Promise(resolve => setTimeout(resolve, 50))

      // ストレージに保存されることを確認
      expect(mockChrome.storage.local.set).toHaveBeenCalledWith({
        extension_config: expect.objectContaining({
          auto_sync: false,
        }),
      })
    })

    it('通知設定チェックボックスの変更が保存される', async () => {
      const notificationsCheckbox = document.getElementById('notifications-checkbox') as HTMLInputElement
      expect(notificationsCheckbox).toBeTruthy()
      expect(notificationsCheckbox.checked).toBe(true)

      // チェックを外す
      notificationsCheckbox.checked = false
      notificationsCheckbox.dispatchEvent(new Event('change'))

      await new Promise(resolve => setTimeout(resolve, 50))

      // ストレージに保存されることを確認
      expect(mockChrome.storage.local.set).toHaveBeenCalledWith({
        extension_config: expect.objectContaining({
          show_notifications: false,
        }),
      })
    })

    it('デバッグモードチェックボックスの変更でデバッグナビが表示/非表示になる', async () => {
      const debugModeCheckbox = document.getElementById('debug-mode-checkbox') as HTMLInputElement
      const debugNavItem = document.getElementById('debug-nav-item')

      expect(debugModeCheckbox).toBeTruthy()
      expect(debugNavItem).toBeTruthy()
      expect(debugModeCheckbox.checked).toBe(false)
      expect(debugNavItem?.style.display).toBe('none')

      // デバッグモードを有効にする
      debugModeCheckbox.checked = true
      debugModeCheckbox.dispatchEvent(new Event('change'))

      await new Promise(resolve => setTimeout(resolve, 50))

      expect(debugNavItem?.style.display).toBe('flex')
    })
  })

  describe('手動同期', () => {
    beforeEach(async () => {
      _controller = new PopupController()
      await new Promise(resolve => setTimeout(resolve, 100))
    })

    it('dMMGamesページで手動同期が成功する', async () => {
      const syncBtn = document.getElementById('manual-sync-btn') as HTMLButtonElement
      expect(syncBtn).toBeTruthy()

      // クリックイベントを発火
      syncBtn.click()

      await new Promise(resolve => setTimeout(resolve, 100))

      // タブ情報の取得を確認
      expect(mockChrome.tabs.query).toHaveBeenCalledWith({
        active: true,
        currentWindow: true,
      })

      // Content Scriptへのメッセージ送信を確認
      expect(mockChrome.tabs.sendMessage).toHaveBeenCalledWith(
        1,
        { type: 'manual_sync_request' },
      )
    })

    it('非対応サイトで手動同期がエラーになる', async () => {
      // 非対応サイトのタブを返すようにモック
      mockChrome.tabs.query.mockResolvedValue([{
        id: 1,
        url: 'https://example.com',
        title: 'Example',
      }])

      const syncBtn = document.getElementById('manual-sync-btn') as HTMLButtonElement
      syncBtn.click()

      await new Promise(resolve => setTimeout(resolve, 100))

      // エラーログが追加されることを確認
      const logContainer = document.getElementById('log-container')
      expect(logContainer?.textContent).toContain('DMM GamesまたはDLsiteのページで実行してください')
    })
  })

  describe('ナビゲーション', () => {
    beforeEach(async () => {
      _controller = new PopupController()
      await new Promise(resolve => setTimeout(resolve, 100))
    })

    it('ナビゲーションボタンでページが切り替わる', async () => {
      const mainPage = document.getElementById('main-page')
      const logsPage = document.getElementById('logs-page')
      const logsNavItem = document.querySelector('[data-page="logs"]')

      // 初期状態
      expect(mainPage?.classList.contains('active')).toBe(true)
      expect(logsPage?.classList.contains('active')).toBe(false)

      // ログページに切り替え
      ;(logsNavItem as HTMLElement)?.click()

      expect(mainPage?.classList.contains('active')).toBe(false)
      expect(logsPage?.classList.contains('active')).toBe(true)
    })
  })

  describe('ログ管理', () => {
    beforeEach(async () => {
      // 初期ログを設定
      mockChrome.storage.local.get.mockResolvedValue({
        extension_config: {
          auto_sync: true,
          show_notifications: true,
          debug_mode: false,
          sync_interval: 30,
        },
        sync_logs: [
          {
            timestamp: new Date().toISOString(),
            level: 'info',
            message: 'テストログ',
          },
        ],
      })

      _controller = new PopupController()
      await new Promise(resolve => setTimeout(resolve, 100))
    })

    it('ログのクリアボタンが機能する', async () => {
      const clearLogBtn = document.getElementById('clear-log-btn')
      const logContainer = document.getElementById('log-container')

      // ログが表示されていることを確認
      expect(logContainer?.textContent).toContain('テストログ')

      // クリアボタンをクリック
      clearLogBtn?.click()

      await new Promise(resolve => setTimeout(resolve, 50))

      // ログがクリアされたことを確認
      const currentLogs = logContainer?.textContent || ''
      expect(currentLogs).toContain('ログをクリアしました')
    })
  })

  describe('デバッグ機能', () => {
    beforeEach(async () => {
      // デバッグモードを有効にした設定でモック
      mockChrome.storage.local.get.mockResolvedValue({
        extension_config: {
          auto_sync: true,
          show_notifications: true,
          debug_mode: true,
          sync_interval: 30,
        },
        sync_logs: [],
      })

      _controller = new PopupController()
      await new Promise(resolve => setTimeout(resolve, 100))

      // デバッグページに切り替え
      const debugNavItem = document.querySelector('[data-page="debug"]') as HTMLElement
      debugNavItem?.click()
    })

    it('jSONメッセージの送信が成功する', async () => {
      const jsonInput = document.getElementById('debug-json-input') as HTMLTextAreaElement
      const sendBtn = document.getElementById('send-json-btn') as HTMLButtonElement
      const responseDiv = document.getElementById('debug-response')

      // JSONを入力
      jsonInput.value = JSON.stringify({ type: 'test', payload: {} }, null, 2)

      // 送信ボタンをクリック
      sendBtn.click()

      await new Promise(resolve => setTimeout(resolve, 100))

      // レスポンスが表示されることを確認
      expect(responseDiv?.textContent).toContain('成功')
    })

    it('無効なJSONでエラーが表示される', async () => {
      const jsonInput = document.getElementById('debug-json-input') as HTMLTextAreaElement
      const sendBtn = document.getElementById('send-json-btn') as HTMLButtonElement
      const responseDiv = document.getElementById('debug-response')

      // 無効なJSONを入力
      jsonInput.value = '{ invalid json'

      // 送信ボタンをクリック
      sendBtn.click()

      await new Promise(resolve => setTimeout(resolve, 50))

      // エラーが表示されることを確認
      expect(responseDiv?.textContent).toContain('無効なJSON')
    })

    it('テンプレートの読み込みが機能する', async () => {
      const jsonInput = document.getElementById('debug-json-input') as HTMLTextAreaElement
      const healthCheckBtn = document.querySelector('[data-template="health_check"]') as HTMLElement

      // テンプレートボタンをクリック
      healthCheckBtn?.click()

      // JSONが設定されることを確認
      const inputValue = JSON.parse(jsonInput.value)
      expect(inputValue.type).toBe('health_check')
      expect(inputValue.payload).toEqual({})
    })

    it('レスポンスのコピーが機能する', async () => {
      const writeTextMock = vi.fn().mockResolvedValue(undefined)
      Object.defineProperty(navigator, 'clipboard', {
        value: { writeText: writeTextMock },
        writable: true,
      })

      const jsonInput = document.getElementById('debug-json-input') as HTMLTextAreaElement
      const sendBtn = document.getElementById('send-json-btn') as HTMLButtonElement
      const copyBtn = document.getElementById('copy-response-btn') as HTMLButtonElement

      // JSONを送信
      jsonInput.value = JSON.stringify({ type: 'test' }, null, 2)
      sendBtn.click()

      await new Promise(resolve => setTimeout(resolve, 100))

      // コピーボタンをクリック
      copyBtn.click()

      await new Promise(resolve => setTimeout(resolve, 50))

      // クリップボードへのコピーが呼ばれたことを確認
      expect(writeTextMock).toHaveBeenCalled()
    })
  })
})
