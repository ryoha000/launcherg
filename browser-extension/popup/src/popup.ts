// ポップアップUIの制御スクリプト

// Extension Internal protobuf types
import type {
  ExtensionRequest,
  StatusData,
} from '@launcherg/shared/proto/extension_internal'

import { create, fromJson, toJson } from '@bufbuild/protobuf'
import { logger } from '@launcherg/shared'

import {
  DebugNativeMessageRequestSchema,
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GetStatusRequestSchema,
} from '@launcherg/shared/proto/extension_internal'

interface ExtensionConfig {
  auto_sync: boolean
  show_notifications: boolean
  debug_mode: boolean
  sync_interval: number
}

// SyncStatusはprotobufのStatusDataを使用するため削除
const log = logger('popup')

interface LogEntry {
  timestamp: string
  level: 'success' | 'error' | 'info'
  message: string
}

export class PopupController {
  private config: ExtensionConfig = {
    auto_sync: true,
    show_notifications: true,
    debug_mode: false,
    sync_interval: 30,
  }

  private logs: LogEntry[] = []
  private maxLogs = 50
  private currentPage = 'main'

  constructor() {
    this.initializeElements()
    this.loadStoredData()
    this.setupEventListeners()
    this.updateStatus()
    this.setupNavigation()
    this.setupDebugSection()
  }

  private initializeElements(): void {
    // 必要な要素の存在確認
    const requiredElements = [
      'connection-status',
      'last-sync',
      'manual-sync-btn',
      'total-synced',
      'success-count',
      'error-count',
      'auto-sync-checkbox',
      'notifications-checkbox',
      'debug-mode-checkbox',
      'log-container',
      'clear-log-btn',
      'loading-overlay',
    ]

    for (const id of requiredElements) {
      if (!document.getElementById(id)) {
        log.warn(`Required element not found: ${id}`)
      }
    }
  }

  private async loadStoredData(): Promise<void> {
    try {
      // 設定を読み込み
      const stored = await chrome.storage.local.get([
        'extension_config',
        'sync_logs',
      ])

      if (stored.extension_config) {
        this.config = { ...this.config, ...stored.extension_config }
        this.updateConfigUI()
      }

      if (stored.sync_logs) {
        this.logs = stored.sync_logs.slice(-this.maxLogs)
        this.updateLogDisplay()
      }
    }
    catch (error) {
      log.error('Failed to load stored data:', error)
      this.addLog('error', 'ストレージからのデータ読み込みに失敗しました')
    }
  }

  private setupEventListeners(): void {
    // 手動同期ボタン
    const syncBtn = document.getElementById(
      'manual-sync-btn',
    ) as HTMLButtonElement
    syncBtn?.addEventListener('click', () => this.performManualSync())

    // 設定チェックボックス
    const autoSyncCheckbox = document.getElementById(
      'auto-sync-checkbox',
    ) as HTMLInputElement
    autoSyncCheckbox?.addEventListener('change', (e) => {
      this.config.auto_sync = (e.target as HTMLInputElement).checked
      this.saveConfig()
    })

    const notificationsCheckbox = document.getElementById(
      'notifications-checkbox',
    ) as HTMLInputElement
    notificationsCheckbox?.addEventListener('change', (e) => {
      this.config.show_notifications = (e.target as HTMLInputElement).checked
      this.saveConfig()
    })

    const debugModeCheckbox = document.getElementById(
      'debug-mode-checkbox',
    ) as HTMLInputElement
    debugModeCheckbox?.addEventListener('change', (e) => {
      this.config.debug_mode = (e.target as HTMLInputElement).checked
      this.updateDebugNavVisibility()
      this.saveConfig()
    })

    // ログクリアボタン
    const clearLogBtn = document.getElementById('clear-log-btn')
    clearLogBtn?.addEventListener('click', () => this.clearLogs())

    // カスタムルール管理ボタン
    const manageRulesBtn = document.getElementById('manage-rules-btn')
    manageRulesBtn?.addEventListener('click', () => this.openRuleManager())

    // ヘルプリンク
    const helpLink = document.getElementById('help-link')
    helpLink?.addEventListener('click', (e) => {
      e.preventDefault()
      this.openHelpPage()
    })

    // モーダル関連
    const modalClose = document.getElementById('modal-close')
    const modalCancel = document.getElementById('modal-cancel')
    const modalOverlay = document.getElementById('modal-overlay')

    modalClose?.addEventListener('click', () => this.closeModal())
    modalCancel?.addEventListener('click', () => this.closeModal())
    modalOverlay?.addEventListener('click', (e) => {
      if (e.target === modalOverlay) {
        this.closeModal()
      }
    })
  }

  private setupNavigation(): void {
    // ナビゲーションボタンのイベントリスナー
    const navButtons = document.querySelectorAll('.nav-item')
    navButtons.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const page = (e.currentTarget as HTMLElement).dataset.page
        if (page) {
          this.switchPage(page)
        }
      })
    })
  }

  private switchPage(pageId: string): void {
    // 現在のページを非表示
    const currentPageElement = document.getElementById(
      `${this.currentPage}-page`,
    )
    const currentNavItem = document.querySelector(
      `[data-page="${this.currentPage}"]`,
    )

    if (currentPageElement) {
      currentPageElement.classList.remove('active')
    }
    if (currentNavItem) {
      currentNavItem.classList.remove('active')
    }

    // 新しいページを表示
    const newPageElement = document.getElementById(`${pageId}-page`)
    const newNavItem = document.querySelector(`[data-page="${pageId}"]`)

    if (newPageElement) {
      newPageElement.classList.add('active')
    }
    if (newNavItem) {
      newNavItem.classList.add('active')
    }

    this.currentPage = pageId

    // ページ切り替え時に統計情報を更新
    if (pageId === 'logs') {
      this.updateDetailedStats()
    }
  }

  private updateDetailedStats(): void {
    // メイン統計と同じ値を詳細統計にも反映
    const totalSynced
      = document.getElementById('total-synced')?.textContent || '0'
    const successCount
      = document.getElementById('success-count')?.textContent || '0'
    const errorCount
      = document.getElementById('error-count')?.textContent || '0'

    const totalSyncedDetailed = document.getElementById(
      'total-synced-detailed',
    )
    const successCountDetailed = document.getElementById(
      'success-count-detailed',
    )
    const errorCountDetailed = document.getElementById('error-count-detailed')

    if (totalSyncedDetailed)
      totalSyncedDetailed.textContent = totalSynced
    if (successCountDetailed)
      successCountDetailed.textContent = successCount
    if (errorCountDetailed)
      errorCountDetailed.textContent = errorCount

    // 成功率を計算
    const total = Number.parseInt(totalSynced)
    const success = Number.parseInt(successCount)
    const successRate = total > 0 ? Math.round((success / total) * 100) : 0

    const successRateElement = document.getElementById('success-rate')
    if (successRateElement) {
      successRateElement.textContent = `${successRate}%`
    }

    // DMM・DLsiteの個別カウント（ダミーデータ、実際は実装時に修正）
    const dmmCount = document.getElementById('dmm-count')
    const dlsiteCount = document.getElementById('dlsite-count')
    if (dmmCount) {
      dmmCount.textContent = Math.floor(
        Number.parseInt(totalSynced) * 0.6,
      ).toString()
    }
    if (dlsiteCount) {
      dlsiteCount.textContent = Math.floor(
        Number.parseInt(totalSynced) * 0.4,
      ).toString()
    }
  }

  private setupDebugSection(): void {
    // デバッグナビゲーションの表示/非表示を設定から読み込み
    this.updateDebugNavVisibility()

    // JSON送信ボタン
    const sendJsonBtn = document.getElementById('send-json-btn')
    sendJsonBtn?.addEventListener('click', () => this.sendDebugJson())

    // クリアボタン
    const clearDebugBtn = document.getElementById('clear-debug-btn')
    clearDebugBtn?.addEventListener('click', () => this.clearDebugInputs())

    // レスポンスコピーボタン
    const copyResponseBtn = document.getElementById('copy-response-btn')
    copyResponseBtn?.addEventListener('click', () => this.copyResponse())

    // テンプレートボタン
    const templateButtons = document.querySelectorAll('.template-btn')
    templateButtons.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        const template = (e.target as HTMLElement).dataset.template
        if (template) {
          this.loadTemplate(template)
        }
      })
    })
  }

  private updateDebugNavVisibility(): void {
    const debugNavItem = document.getElementById('debug-nav-item')
    if (debugNavItem) {
      debugNavItem.style.display = this.config.debug_mode ? 'flex' : 'none'
    }

    // デバッグモードが無効になった場合、メインページに戻る
    if (!this.config.debug_mode && this.currentPage === 'debug') {
      this.switchPage('main')
    }
  }

  private async sendDebugJson(): Promise<void> {
    const jsonInput = document.getElementById(
      'debug-json-input',
    ) as HTMLTextAreaElement
    const responseDiv = document.getElementById('debug-response')
    const sendBtn = document.getElementById(
      'send-json-btn',
    ) as HTMLButtonElement

    if (!jsonInput || !responseDiv || !sendBtn)
      return

    const jsonText = jsonInput.value.trim()
    if (!jsonText) {
      this.displayDebugResponse(
        { error: 'JSONメッセージが入力されていません' },
        true,
      )
      return
    }

    // JSONの妥当性チェック
    let parsedMessage
    try {
      parsedMessage = JSON.parse(jsonText)
    }
    catch (e) {
      this.displayDebugResponse({ error: `無効なJSON: ${e}` }, true)
      return
    }

    // UI状態を更新
    sendBtn.disabled = true
    sendBtn.textContent = '送信中...'
    this.displayDebugResponse({ message: '送信中...' }, false)

    try {
      // プロトバフメッセージを作成
      const request = create(ExtensionRequestSchema, {
        requestId: this.generateRequestId(),
        request: {
          case: 'debugNativeMessage',
          value: create(DebugNativeMessageRequestSchema, {
            payloadJson: JSON.stringify(parsedMessage),
          }),
        },
      })

      // バックグラウンドスクリプトに送信
      const responseJson = await this.sendProtobufMessage(request)
      const response = fromJson(ExtensionResponseSchema, responseJson)

      this.displayDebugResponse(response, !response.success)

      if (response.success) {
        this.addLog(
          'success',
          `デバッグメッセージ送信成功: ${parsedMessage.type || 'unknown'}`,
        )
      }
      else {
        this.addLog('error', `デバッグメッセージ送信失敗: ${response.error}`)
      }
    }
    catch (error) {
      const errorResponse = {
        error: error instanceof Error ? error.message : 'Unknown error',
        details: 'バックグラウンドスクリプトとの通信に失敗しました',
      }
      this.displayDebugResponse(errorResponse, true)
      this.addLog(
        'error',
        `デバッグメッセージ通信エラー: ${errorResponse.error}`,
      )
    }
    finally {
      // UI状態を復元
      sendBtn.disabled = false
      sendBtn.textContent = 'JSON送信'
    }
  }

  private displayDebugResponse(response: any, isError: boolean): void {
    const responseDiv = document.getElementById('debug-response')
    if (!responseDiv)
      return

    // JSONを整形して表示
    const formattedJson = JSON.stringify(response, null, 2)

    responseDiv.innerHTML = `
      <div class="debug-response-header ${isError ? 'error' : 'success'}">
        <span class="response-status">${
          isError ? '❌ エラー' : '✅ 成功'
        }</span>
        <span class="response-time">${new Date().toLocaleTimeString(
          'ja-JP',
        )}</span>
      </div>
      <pre class="debug-response-content"><code>${this.escapeHtml(
        formattedJson,
      )}</code></pre>
    `
  }

  private clearDebugInputs(): void {
    const jsonInput = document.getElementById(
      'debug-json-input',
    ) as HTMLTextAreaElement
    const responseDiv = document.getElementById('debug-response')

    if (jsonInput) {
      jsonInput.value = ''
    }
    if (responseDiv) {
      responseDiv.textContent = 'レスポンスがここに表示されます'
      responseDiv.className = 'debug-response'
    }
  }

  private async copyResponse(): Promise<void> {
    const responseDiv = document.getElementById('debug-response')
    const copyBtn = document.getElementById('copy-response-btn')
    if (!responseDiv || !copyBtn)
      return

    const codeElement = responseDiv.querySelector('code')
    if (!codeElement)
      return

    try {
      await navigator.clipboard.writeText(codeElement.textContent || '')

      const originalText = copyBtn.textContent
      copyBtn.textContent = '✅'
      setTimeout(() => {
        copyBtn.textContent = originalText
      }, 1000)

      this.addLog('info', 'デバッグレスポンスをクリップボードにコピーしました')
    }
    catch (error) {
      this.addLog(
        'error',
        `クリップボードへのコピーに失敗しました。 ${
          error instanceof Error ? error.message : 'Unknown error'
        }`,
      )
    }
  }

  private loadTemplate(templateType: string): void {
    const jsonInput = document.getElementById(
      'debug-json-input',
    ) as HTMLTextAreaElement
    if (!jsonInput)
      return

    const templates = {
      health_check: {
        type: 'health_check',
        payload: {},
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      },
      get_status: {
        type: 'get_status',
        payload: {},
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      },
      sync_games: {
        type: 'sync_games',
        payload: {
          store: 'DMM',
          extension_id: chrome.runtime.id,
          games: [
            {
              store_id: 'test_game_123',
              title: 'テストゲーム',
              purchase_url: 'https://games.dmm.co.jp/game/test_game_123',
              purchase_date: '2025-01-30',
              thumbnail_url: 'https://example.com/thumbnail.jpg',
              additional_data: {
                store_name: 'DMM Games',
                extraction_source: 'debug-test',
                extraction_timestamp: new Date().toISOString(),
              },
            },
          ],
        },
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      },
      set_config: {
        type: 'set_config',
        payload: {
          auto_sync: true,
          allowed_domains: ['games.dmm.co.jp', 'www.dlsite.com'],
          sync_interval_minutes: 30,
          debug_mode: true,
        },
        timestamp: new Date().toISOString(),
        request_id: this.generateRequestId(),
      },
    }

    const template = templates[templateType as keyof typeof templates]
    if (template) {
      jsonInput.value = JSON.stringify(template, null, 2)
      this.addLog('info', `テンプレート「${templateType}」を読み込みました`)
    }
  }

  private generateRequestId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2)
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div')
    div.textContent = text
    return div.innerHTML
  }

  private async updateStatus(): Promise<void> {
    try {
      // プロトバフメッセージを作成
      const request = create(ExtensionRequestSchema, {
        requestId: this.generateRequestId(),
        request: {
          case: 'getStatus',
          value: create(GetStatusRequestSchema, {}),
        },
      })

      // バックグラウンドスクリプトからステータスを取得
      const responseJson = await this.sendProtobufMessage(request)
      const response = fromJson(ExtensionResponseSchema, responseJson)

      if (
        response
        && response.success
        && response.response.case === 'statusResult'
      ) {
        this.updateStatusDisplay(response.response.value.status)
        this.updateConnectionStatus('connected')
      }
      else {
        this.updateConnectionStatus('disconnected')
      }
    }
    catch (error) {
      log.error('Failed to get status:', error)
      this.updateConnectionStatus('disconnected')
      this.addLog('error', 'ステータスの取得に失敗しました')
    }
  }

  private updateStatusDisplay(status?: StatusData): void {
    if (!status)
      return

    // 最終同期時刻
    const lastSyncElement = document.getElementById('last-sync')
    if (lastSyncElement && status.lastSync) {
      const lastSync = new Date(status.lastSync)
      lastSyncElement.textContent = this.formatDate(lastSync)
    }

    // 統計情報
    const totalSyncedElement = document.getElementById('total-synced')
    if (totalSyncedElement) {
      totalSyncedElement.textContent = status.totalSynced.toString()
    }
  }

  private updateConnectionStatus(status: 'connected' | 'disconnected'): void {
    const statusElement = document.getElementById('connection-status')
    if (statusElement) {
      statusElement.className = `status-value ${status}`
      statusElement.textContent = status === 'connected' ? '接続済み' : '切断'
    }
  }

  private updateConfigUI(): void {
    const autoSyncCheckbox = document.getElementById(
      'auto-sync-checkbox',
    ) as HTMLInputElement
    const notificationsCheckbox = document.getElementById(
      'notifications-checkbox',
    ) as HTMLInputElement
    const debugModeCheckbox = document.getElementById(
      'debug-mode-checkbox',
    ) as HTMLInputElement

    if (autoSyncCheckbox)
      autoSyncCheckbox.checked = this.config.auto_sync
    if (notificationsCheckbox)
      notificationsCheckbox.checked = this.config.show_notifications
    if (debugModeCheckbox)
      debugModeCheckbox.checked = this.config.debug_mode
  }

  private async performManualSync(): Promise<void> {
    const syncBtn = document.getElementById(
      'manual-sync-btn',
    ) as HTMLButtonElement
    const loadingOverlay = document.getElementById('loading-overlay')

    try {
      // UIを無効化
      if (syncBtn) {
        syncBtn.disabled = true
        syncBtn.textContent = '同期中...'
      }

      loadingOverlay?.classList.remove('hidden')

      // 現在のアクティブタブを取得
      const tabs = await chrome.tabs.query({
        active: true,
        currentWindow: true,
      })
      const currentTab = tabs[0]

      if (!currentTab?.id) {
        throw new Error('アクティブなタブが見つかりません')
      }

      // タブのURLをチェック
      const isDMMGames = currentTab.url?.includes('games.dmm.co.jp')
      const isDLsite = currentTab.url?.includes('dlsite.com')

      if (!isDMMGames && !isDLsite) {
        throw new Error('DMM GamesまたはDLsiteのページで実行してください')
      }

      // Content Scriptに同期要求を送信（受信側がいない場合は注入して再試行）
      let response: any
      try {
        response = await chrome.tabs.sendMessage(currentTab.id, {
          type: 'manual_sync_request',
        })
      }
      catch (err) {
        const msg = err instanceof Error ? err.message : String(err)
        if (/Receiving end does not exist/i.test(msg)) {
          const files: string[] = []
          if (isDMMGames)
            files.push('content-scripts/dmm-extractor.js')
          if (isDLsite)
            files.push('content-scripts/dlsite-extractor.js')
          if (files.length > 0) {
            await chrome.scripting.executeScript({
              target: { tabId: currentTab.id },
              files,
            })
            response = await chrome.tabs.sendMessage(currentTab.id, {
              type: 'manual_sync_request',
            })
          }
        }
        else {
          throw err
        }
      }

      if (response && response.success) {
        this.addLog('success', `手動同期が完了しました: ${response.message}`)
        this.updateStatus() // ステータスを更新
      }
      else {
        throw new Error(response?.error || '同期に失敗しました')
      }
    }
    catch (error) {
      log.error('Manual sync failed:', error)
      const errorMessage
        = error instanceof Error ? error.message : 'Unknown error'
      this.addLog('error', `手動同期に失敗: ${errorMessage}`)
    }
    finally {
      // UIを復元
      if (syncBtn) {
        syncBtn.disabled = false
        syncBtn.innerHTML = '<span class="button-icon">🔄</span>手動同期'
      }

      loadingOverlay?.classList.add('hidden')
    }
  }

  private async saveConfig(): Promise<void> {
    try {
      await chrome.storage.local.set({ extension_config: this.config })

      // バックグラウンドスクリプトに設定変更を通知（暫定的にsendMessageを使用）
      await this.sendMessage({
        type: 'config_updated',
        config: this.config,
      })

      this.addLog('info', '設定を保存しました')
    }
    catch (error) {
      log.error('Failed to save config:', error)
      this.addLog('error', '設定の保存に失敗しました')
    }
  }

  private addLog(level: 'success' | 'error' | 'info', message: string): void {
    const log: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
    }

    this.logs.unshift(log)
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(0, this.maxLogs)
    }

    this.updateLogDisplay()
    this.saveLogs()
  }

  private updateLogDisplay(): void {
    const logContainer = document.getElementById('log-container')
    if (!logContainer)
      return

    if (this.logs.length === 0) {
      logContainer.innerHTML
        = '<p class="text-center" style="color: #999; padding: 20px;">ログはありません</p>'
      return
    }

    logContainer.innerHTML = this.logs
      .slice(0, 10) // 最新10件のみ表示
      .map(
        log => `
        <div class="log-entry ${log.level}">
          <span class="log-timestamp">${this.formatTime(
            new Date(log.timestamp),
          )}</span>
          ${log.message}
        </div>
      `,
      )
      .join('')
  }

  private async saveLogs(): Promise<void> {
    try {
      await chrome.storage.local.set({ sync_logs: this.logs })
    }
    catch (error) {
      log.error('Failed to save logs:', error)
    }
  }

  private clearLogs(): void {
    this.logs = []
    this.updateLogDisplay()
    this.saveLogs()
    this.addLog('info', 'ログをクリアしました')
  }

  private openRuleManager(): void {
    // カスタムルール管理モーダルを表示
    const modalTitle = document.getElementById('modal-title')
    const modalBody = document.getElementById('modal-body')
    const modalOverlay = document.getElementById('modal-overlay')

    if (modalTitle && modalBody && modalOverlay) {
      modalTitle.textContent = 'カスタムルール管理'
      modalBody.innerHTML = `
        <p>カスタムルール機能は開発中です。</p>
        <p>サイトのレイアウトが変更された場合の対応方法：</p>
        <ol>
          <li>GitHubのIssuesページで報告</li>
          <li>拡張機能のアップデートを待つ</li>
          <li>一時的に手動でゲーム情報を追加</li>
        </ol>
      `
      modalOverlay.classList.remove('hidden')
    }
  }

  private openHelpPage(): void {
    chrome.tabs.create({
      url: 'https://github.com/your-repo/launcherg-extension/wiki',
    })
  }

  private closeModal(): void {
    const modalOverlay = document.getElementById('modal-overlay')
    modalOverlay?.classList.add('hidden')
  }

  private async sendMessage(message: any): Promise<any> {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(message, (response) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message))
        }
        else {
          resolve(response)
        }
      })
    })
  }

  private async sendProtobufMessage(request: ExtensionRequest): Promise<any> {
    return new Promise((resolve, reject) => {
      // プロトバフメッセージをJSONとしてシリアライズ
      const messageJson = toJson(ExtensionRequestSchema, request)

      chrome.runtime.sendMessage(messageJson, (response) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message))
        }
        else {
          resolve(response)
        }
      })
    })
  }

  private formatDate(date: Date): string {
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / (1000 * 60))
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1)
      return 'たった今'
    if (diffMins < 60)
      return `${diffMins}分前`
    if (diffHours < 24)
      return `${diffHours}時間前`
    if (diffDays < 7)
      return `${diffDays}日前`

    return date.toLocaleDateString('ja-JP')
  }

  private formatTime(date: Date): string {
    return date.toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
    })
  }
}

// DOMが読み込まれたら初期化
// テスト環境では自動初期化しない

if (typeof globalThis !== 'undefined') {
  document.addEventListener('DOMContentLoaded', () => {
    // eslint-disable-next-line no-new
    new PopupController()
  })
}
