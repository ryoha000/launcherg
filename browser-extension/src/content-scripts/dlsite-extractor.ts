import type { ExtractedGameData, SiteConfig } from './base-extractor'
import { BaseExtractor } from './base-extractor'

// DLsite用の設定を読み込み
let dlsiteConfig: SiteConfig

// 設定を動的に読み込み
chrome.runtime.sendMessage({ type: 'get_config', site: 'dlsite' }, (response) => {
  if (response && response.config) {
    dlsiteConfig = response.config
    initDLsiteExtractor()
  }
})

function initDLsiteExtractor() {
  if (!dlsiteConfig) {
    console.error('[DLsite Extractor] Config not loaded')
    return
  }

  const extractor = new DLsiteExtractor(dlsiteConfig)

  if (extractor.shouldExtract()) {
    console.log('[DLsite Extractor] Starting extraction on DLsite')
    extractor.extractAndSync()
  }
}

class DLsiteExtractor extends BaseExtractor {
  private isExtracting: boolean = false

  constructor(config: SiteConfig) {
    super(config, true) // デバッグモード有効
  }

  shouldExtract(): boolean {
    // ページURL確認
    if (!window.location.hostname.includes('dlsite.com')) {
      return false
    }

    // 購入済み作品ページかどうか確認
    const isDLsiteLibrary = window.location.pathname.includes('/library')
      || window.location.pathname.includes('/mypage')
      || window.location.search.includes('purchase')

    if (!isDLsiteLibrary) {
      return false
    }

    // 検出ルールによる確認
    return this.detectPage()
  }

  async extractAndSync(): Promise<void> {
    if (this.isExtracting) {
      console.log('[DLsite Extractor] Already extracting, skipping')
      return
    }

    this.isExtracting = true

    try {
      // ページが完全に読み込まれるまで待機
      await this.waitForPageLoad()

      // ゲーム情報を抽出
      const games = this.extractGames()

      if (games.length === 0) {
        console.log('[DLsite Extractor] No games found')
        return
      }

      console.log(`[DLsite Extractor] Found ${games.length} games`)

      // DLsite特有の処理
      const processedGames = games.map(game => this.processDLsiteGame(game))

      // バックグラウンドスクリプトに送信
      chrome.runtime.sendMessage({
        type: 'sync_games',
        store: 'DLSite',
        games: processedGames,
        source: 'dlsite-extractor',
      }, (response) => {
        if (response && response.success) {
          console.log('[DLsite Extractor] Sync successful:', response)
          this.showNotification(`DLsite: ${processedGames.length}個の作品を同期しました`)
        }
        else {
          console.error('[DLsite Extractor] Sync failed:', response)
          this.showNotification('DLsite: 同期に失敗しました', 'error')
        }
      })
    }
    catch (error) {
      console.error('[DLsite Extractor] Extraction failed:', error)
      this.showNotification('DLsite: 作品情報の抽出に失敗しました', 'error')
    }
    finally {
      this.isExtracting = false
    }
  }

  private processDLsiteGame(game: ExtractedGameData): ExtractedGameData {
    // DLsiteのURLを正規化
    if (game.purchase_url && !game.purchase_url.startsWith('http')) {
      game.purchase_url = `https://www.dlsite.com${game.purchase_url}`
    }

    // DLsite特有のstore_id処理（RJ/VJ/BJ codes）
    if (game.store_id) {
      // URLから作品コードを抽出
      const match = game.purchase_url.match(/\/(RJ|VJ|BJ)(\d+)/)
      if (match) {
        game.store_id = match[1] + match[2]
      }
      // 既存のstore_idが正しい形式かチェック
      else if (!game.store_id.match(/^(RJ|VJ|BJ)\d+$/)) {
        // 数字のみの場合はRJを付加（最も一般的）
        if (game.store_id.match(/^\d+$/)) {
          game.store_id = `RJ${game.store_id}`
        }
      }
    }

    // サムネイルURLの正規化
    if (game.thumbnail_url) {
      if (game.thumbnail_url.startsWith('//')) {
        game.thumbnail_url = `https:${game.thumbnail_url}`
      }
      else if (!game.thumbnail_url.startsWith('http')) {
        game.thumbnail_url = `https://www.dlsite.com${game.thumbnail_url}`
      }
    }

    // 購入日の正規化（DLsiteの日付フォーマット対応）
    if (game.purchase_date) {
      game.purchase_date = this.normalizeDLsiteDate(game.purchase_date)
    }

    // タイトルのクリーンアップ（DLsiteの不要な文字列を除去）
    if (game.title) {
      game.title = this.cleanDLsiteTitle(game.title)
    }

    // DLsite特有の追加情報
    game.additional_data.store_name = 'DLsite'
    game.additional_data.extraction_source = 'dlsite-extractor'
    game.additional_data.extraction_timestamp = new Date().toISOString()

    // 作品の種類を判定
    if (game.store_id.startsWith('RJ')) {
      game.additional_data.work_type = 'doujin'
    }
    else if (game.store_id.startsWith('VJ')) {
      game.additional_data.work_type = 'voice'
    }
    else if (game.store_id.startsWith('BJ')) {
      game.additional_data.work_type = 'book'
    }

    return game
  }

  private normalizeDLsiteDate(dateStr: string): string {
    try {
      // DLsite日付フォーマット対応: "YYYY年MM月DD日", "YYYY/MM/DD", "YYYY-MM-DD"
      let cleanDate = dateStr
        .replace(/年/g, '/')
        .replace(/月/g, '/')
        .replace(/日/g, '')
        .replace(/\s+/g, '')

      const date = new Date(cleanDate)
      return date.toISOString().split('T')[0]
    }
    catch {
      return dateStr
    }
  }

  private cleanDLsiteTitle(title: string): string {
    // DLsiteのタイトルから不要な情報を除去
    return title
      .replace(/\[.*?\]/g, '') // [サークル名] などを除去
      .replace(/（.*?）/g, '') // 全角括弧の内容を除去
      .replace(/\(.*?\)/g, '') // 半角括弧の内容を除去
      .replace(/\s+/g, ' ') // 連続する空白を単一の空白に
      .trim()
  }

  private async waitForPageLoad(): Promise<void> {
    return new Promise((resolve) => {
      if (document.readyState === 'complete') {
        // DLsiteは動的コンテンツが多いので少し長めに待機
        setTimeout(resolve, 2000)
      }
      else {
        window.addEventListener('load', () => {
          setTimeout(resolve, 2000)
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

    // ページ内通知も表示
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

    // 4秒後に自動削除
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification)
      }
    }, 4000)
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
      if (dlsiteConfig) {
        initDLsiteExtractor()
      }
    }, 2000)
  }
})

observer.observe(document.body, {
  childList: true,
  subtree: true,
})

console.log('[DLsite Extractor] Script loaded')
