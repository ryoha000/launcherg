import type { ExtractedGameData, SiteConfig } from './base-extractor'
import { BaseExtractor } from './base-extractor'

// DMM Games用の設定を読み込み
let dmmConfig: SiteConfig

// 設定を動的に読み込み
chrome.runtime.sendMessage({ type: 'get_config', site: 'dmm' }, (response) => {
  if (response && response.config) {
    dmmConfig = response.config
    initDMMExtractor()
  }
})

function initDMMExtractor() {
  if (!dmmConfig) {
    console.error('[DMM Extractor] Config not loaded')
    return
  }

  const extractor = new DMMExtractor(dmmConfig)

  if (extractor.shouldExtract()) {
    console.log('[DMM Extractor] Starting extraction on DMM Games')
    extractor.extractAndSync()
  }
}

class DMMExtractor extends BaseExtractor {
  private isExtracting: boolean = false

  constructor(config: SiteConfig) {
    super(config, true) // デバッグモード有効
  }

  shouldExtract(): boolean {
    // ページURL確認
    if (!window.location.hostname.includes('games.dmm.co.jp')) {
      return false
    }

    // 検出ルールによる確認
    return this.detectPage()
  }

  async extractAndSync(): Promise<void> {
    if (this.isExtracting) {
      console.log('[DMM Extractor] Already extracting, skipping')
      return
    }

    this.isExtracting = true

    try {
      // ページが完全に読み込まれるまで待機
      await this.waitForPageLoad()

      // ゲーム情報を抽出
      const games = this.extractGames()

      if (games.length === 0) {
        console.log('[DMM Extractor] No games found')
        return
      }

      console.log(`[DMM Extractor] Found ${games.length} games`)

      // DMM特有の処理
      const processedGames = games.map(game => this.processDMMGame(game))

      // バックグラウンドスクリプトに送信
      chrome.runtime.sendMessage({
        type: 'sync_games',
        store: 'DMM',
        games: processedGames,
        source: 'dmm-extractor',
      }, (response) => {
        if (response && response.success) {
          console.log('[DMM Extractor] Sync successful:', response)
          this.showNotification(`DMM: ${processedGames.length}個のゲームを同期しました`)
        }
        else {
          console.error('[DMM Extractor] Sync failed:', response)
          this.showNotification('DMM: 同期に失敗しました', 'error')
        }
      })
    }
    catch (error) {
      console.error('[DMM Extractor] Extraction failed:', error)
      this.showNotification('DMM: ゲーム情報の抽出に失敗しました', 'error')
    }
    finally {
      this.isExtracting = false
    }
  }

  private processDMMGame(game: ExtractedGameData): ExtractedGameData {
    // DMMのURLを正規化
    if (game.purchase_url && !game.purchase_url.startsWith('http')) {
      game.purchase_url = `https://games.dmm.co.jp${game.purchase_url}`
    }

    // DMM特有のstore_id処理
    if (game.store_id && game.store_id.includes('_')) {
      // 例: "game_12345" -> "12345"
      game.store_id = game.store_id.split('_').pop() || game.store_id
    }

    // サムネイルURLの正規化
    if (game.thumbnail_url && !game.thumbnail_url.startsWith('http')) {
      game.thumbnail_url = `https:${game.thumbnail_url}`
    }

    // 購入日の正規化（DMMの日付フォーマット対応）
    if (game.purchase_date) {
      game.purchase_date = this.normalizeDMMDate(game.purchase_date)
    }

    // DMM特有の追加情報
    game.additional_data.store_name = 'DMM Games'
    game.additional_data.extraction_source = 'dmm-extractor'
    game.additional_data.extraction_timestamp = new Date().toISOString()

    return game
  }

  private normalizeDMMDate(dateStr: string): string {
    try {
      // DMM日付フォーマット: "YYYY/MM/DD" or "YYYY-MM-DD"
      const cleanDate = dateStr.replace(/[年月日]/g, '/').replace(/\s+/g, '')
      const date = new Date(cleanDate)
      return date.toISOString().split('T')[0] // YYYY-MM-DD形式で返す
    }
    catch {
      return dateStr // 変換できない場合はそのまま返す
    }
  }

  private async waitForPageLoad(): Promise<void> {
    return new Promise((resolve) => {
      if (document.readyState === 'complete') {
        // 追加で少し待機（動的コンテンツの読み込み待ち）
        setTimeout(resolve, 1000)
      }
      else {
        window.addEventListener('load', () => {
          setTimeout(resolve, 1000)
        })
      }
    })
  }

  private showNotification(message: string, type: 'success' | 'error' = 'success'): void {
    // ブラウザ通知を表示
    chrome.runtime.sendMessage({
      type: 'show_notification',
      title: 'Launcherg DL Store Sync',
      message,
      iconType: type,
    })

    // ページ内通知も表示（オプション）
    this.showInPageNotification(message, type)
  }

  private showInPageNotification(message: string, type: 'success' | 'error'): void {
    const notification = document.createElement('div')
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: ${type === 'success' ? '#4CAF50' : '#f44336'};
      color: white;
      padding: 12px 20px;
      border-radius: 4px;
      z-index: 10000;
      font-family: Arial, sans-serif;
      font-size: 14px;
      box-shadow: 0 2px 10px rgba(0,0,0,0.3);
      animation: slideIn 0.3s ease-out;
    `

    notification.textContent = message
    document.body.appendChild(notification)

    // 3秒後に自動削除
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification)
      }
    }, 3000)
  }
}

// CSS animation
const style = document.createElement('style')
style.textContent = `
  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
`
document.head.appendChild(style)

// ページ変更の監視（SPA対応）
let currentUrl = window.location.href
const observer = new MutationObserver(() => {
  if (window.location.href !== currentUrl) {
    currentUrl = window.location.href
    // URL変更時に再度チェック
    setTimeout(() => {
      if (dmmConfig) {
        initDMMExtractor()
      }
    }, 2000)
  }
})

observer.observe(document.body, {
  childList: true,
  subtree: true,
})

console.log('[DMM Extractor] Script loaded')
