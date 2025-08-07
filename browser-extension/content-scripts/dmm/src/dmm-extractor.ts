// Extension Internal protobuf types

import { create, fromJson, toJson } from '@bufbuild/protobuf'

import {
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GameDataSchema,
  GetConfigRequestSchema,
  ShowNotificationRequestSchema,
  SyncGamesRequestSchema,
} from '@launcherg/shared/proto/extension_internal'

// ゲームデータの型定義
interface ExtractedGameData {
  store_id: string
  title: string
  purchase_url: string
  purchase_date?: string
  thumbnail_url?: string
  additional_data: Record<string, string>
}

// サイト設定の型定義
interface ExtractionRule {
  name?: string
  selectors?: string[]
  selector?: string
  attribute?: string
  fallbackAttribute?: string
  processor?: 'text' | 'html' | 'attr'
  urlPattern?: string
  required: boolean
  description?: string
}

interface SiteConfig {
  name: string
  domain: string
  detectionRules: ExtractionRule[]
  gameExtractionRules: {
    container: string
    fields: Record<string, ExtractionRule>
  }
}

// DMM Games用の設定を読み込み
let dmmConfig: SiteConfig

function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// 設定を動的に読み込みをプロトバフで実行
const getConfigRequest = create(ExtensionRequestSchema, {
  requestId: generateRequestId(),
  request: {
    case: 'getConfig',
    value: create(GetConfigRequestSchema, {
      site: 'dmm',
    }),
  },
})

chrome.runtime.sendMessage(
  toJson(ExtensionRequestSchema, getConfigRequest),
  (responseJson) => {
    try {
      const response = fromJson(ExtensionResponseSchema, responseJson)
      if (
        response
        && response.success
        && response.response.case === 'configResult'
      ) {
        dmmConfig = JSON.parse(response.response.value.configJson)
        initDMMExtractor()
      }
    }
    catch (error) {
      console.error('[DMM Extractor] Failed to parse config response:', error)
    }
  },
)

function initDMMExtractor() {
  if (!dmmConfig) {
    console.error('[DMM Extractor] Config not loaded')
    return
  }

  if (shouldExtract(dmmConfig)) {
    console.log('[DMM Extractor] Starting extraction on DMM Games')
    extractAndSync(dmmConfig)
  }
}

// 純粋関数でページ検出
function detectPage(config: SiteConfig): boolean {
  for (const rule of config.detectionRules) {
    const element = findElement(rule)
    if (rule.required && !element) {
      console.log(`[DMM Extractor] Required detection rule failed: ${rule.name}`)
      return false
    }
    if (element) {
      console.log(`[DMM Extractor] Detection rule matched: ${rule.name}`)
      return true
    }
  }
  return false
}

// 要素検索の純粋関数
function findElement(rule: ExtractionRule, container: HTMLElement | Document = document): Element | null {
  const selectors = rule.selectors || (rule.selector ? [rule.selector] : [])

  for (const selector of selectors) {
    try {
      const element = container.querySelector(selector)
      if (element) {
        return element
      }
    }
    catch (error) {
      console.log(`[DMM Extractor] Invalid selector: ${selector}`, error)
    }
  }

  return null
}

// フィールド抽出の純粋関数
function extractField(container: HTMLElement, rule: ExtractionRule): string | null {
  const element = findElement(rule, container)
  if (!element) {
    return null
  }

  let value: string | null = null

  // 値の抽出方法を決定
  if (rule.attribute) {
    value = element.getAttribute(rule.attribute)
    if (!value && rule.fallbackAttribute) {
      value = element.getAttribute(rule.fallbackAttribute)
    }
  }
  else {
    switch (rule.processor) {
      case 'html':
        value = element.innerHTML
        break
      case 'text':
      default:
        value = element.textContent
        break
    }
  }

  // URLパターンからの抽出
  if (!value && rule.urlPattern && rule.attribute === 'href') {
    const href = element.getAttribute('href')
    if (href) {
      const match = href.match(new RegExp(rule.urlPattern))
      if (match && match[1]) {
        value = match[1]
      }
    }
  }

  // フォールバック属性の使用
  if (!value && rule.fallbackAttribute) {
    value = element.getAttribute(rule.fallbackAttribute)
  }

  return value ? value.trim() : null
}

// 単一ゲーム抽出の純粋関数
function extractSingleGame(container: HTMLElement, config: SiteConfig): ExtractedGameData | null {
  const fields = config.gameExtractionRules.fields
  const gameData: Partial<ExtractedGameData> = {
    additional_data: {},
  }

  // 各フィールドを抽出
  for (const [fieldName, rule] of Object.entries(fields)) {
    try {
      const value = extractField(container, rule)
      if (value) {
        if (fieldName === 'store_id' || fieldName === 'title' || fieldName === 'purchase_url') {
          gameData[fieldName] = value
        }
        else if (fieldName === 'purchase_date' || fieldName === 'thumbnail_url') {
          gameData[fieldName] = value
        }
        else {
          gameData.additional_data![fieldName] = value
        }
      }
      else if (rule.required) {
        console.log(`[DMM Extractor] Required field missing: ${fieldName}`)
        return null
      }
    }
    catch (error) {
      console.log(`[DMM Extractor] Error extracting field ${fieldName}:`, error)
      if (rule.required) {
        return null
      }
    }
  }

  // 必須フィールドの確認
  if (!gameData.store_id || !gameData.title || !gameData.purchase_url) {
    console.log('[DMM Extractor] Missing required fields:', gameData)
    return null
  }

  return gameData as ExtractedGameData
}

// ゲーム情報抽出の純粋関数
function extractGames(config: SiteConfig): ExtractedGameData[] {
  const containers = document.querySelectorAll(config.gameExtractionRules.container)
  console.log(`[DMM Extractor] Found ${containers.length} game containers`)

  const games: ExtractedGameData[] = []

  containers.forEach((container, index) => {
    try {
      const gameData = extractSingleGame(container as HTMLElement, config)
      if (gameData) {
        games.push(gameData)
        console.log(`[DMM Extractor] Extracted game ${index + 1}:`, gameData)
      }
    }
    catch (error) {
      console.log(`[DMM Extractor] Error extracting game ${index + 1}:`, error)
    }
  })

  return games
}

// ページ検出の純粋関数
function shouldExtract(config: SiteConfig): boolean {
  // ページURL確認
  if (!window.location.hostname.includes('games.dmm.co.jp')) {
    return false
  }

  // 検出ルールによる確認
  return detectPage(config)
}

// DMM特有のゲーム処理
function processDMMGame(game: ExtractedGameData): ExtractedGameData {
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
    game.purchase_date = normalizeDMMDate(game.purchase_date)
  }

  // DMM特有の追加情報
  game.additional_data.store_name = 'DMM Games'
  game.additional_data.extraction_source = 'dmm-extractor'
  game.additional_data.extraction_timestamp = new Date().toISOString()

  return game
}

// DMM日付正規化の純粋関数
function normalizeDMMDate(dateStr: string): string {
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

// ページ読み込み待機の純粋関数
function waitForPageLoad(): Promise<void> {
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

// ページ内通知の純粋関数
function showInPageNotification(message: string, type: 'success' | 'error'): void {
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

// 通知の純粋関数
function showNotification(message: string, type: 'success' | 'error' = 'success'): void {
  // プロトバフで通知メッセージを作成
  const notificationRequest = create(ExtensionRequestSchema, {
    requestId: generateRequestId(),
    request: {
      case: 'showNotification',
      value: create(ShowNotificationRequestSchema, {
        title: 'Launcherg DL Store Sync',
        message,
        iconType: type,
      }),
    },
  })

  // ブラウザ通知を表示
  chrome.runtime.sendMessage(
    toJson(ExtensionRequestSchema, notificationRequest),
  )

  // ページ内通知も表示（オプション）
  showInPageNotification(message, type)
}

// 抽出状態管理
let isExtracting = false

// メインの抽出と同期処理
async function extractAndSync(config: SiteConfig): Promise<void> {
  if (isExtracting) {
    console.log('[DMM Extractor] Already extracting, skipping')
    return
  }

  isExtracting = true

  try {
    // ページが完全に読み込まれるまで待機
    await waitForPageLoad()

    // ゲーム情報を抽出
    const games = extractGames(config)

    if (games.length === 0) {
      console.log('[DMM Extractor] No games found')
      return
    }

    console.log(`[DMM Extractor] Found ${games.length} games`)

    // DMM特有の処理
    const processedGames = games.map(game => processDMMGame(game))

    // プロトバフでゲームデータを変換
    const gameDataList = processedGames.map(game =>
      create(GameDataSchema, {
        storeId: game.store_id,
        title: game.title,
        purchaseUrl: game.purchase_url,
        purchaseDate: game.purchase_date || '',
        thumbnailUrl: game.thumbnail_url || '',
        additionalData: game.additional_data,
      }),
    )

    // プロトバフメッセージを作成
    const syncRequest = create(ExtensionRequestSchema, {
      requestId: generateRequestId(),
      request: {
        case: 'syncGames',
        value: create(SyncGamesRequestSchema, {
          store: 'DMM',
          games: gameDataList,
          source: 'dmm-extractor',
        }),
      },
    })

    // バックグラウンドスクリプトに送信
    chrome.runtime.sendMessage(
      toJson(ExtensionRequestSchema, syncRequest),
      (responseJson) => {
        try {
          const response = fromJson(ExtensionResponseSchema, responseJson)
          if (
            response
            && response.success
            && response.response.case === 'syncGamesResult'
          ) {
            console.log('[DMM Extractor] Sync successful:', response)
            showNotification(
              `DMM: ${processedGames.length}個のゲームを同期しました`,
            )
          }
          else {
            console.error('[DMM Extractor] Sync failed:', response)
            showNotification('DMM: 同期に失敗しました', 'error')
          }
        }
        catch (error) {
          console.error(
            '[DMM Extractor] Failed to parse sync response:',
            error,
          )
          showNotification(
            'DMM: 同期レスポンスの解析に失敗しました',
            'error',
          )
        }
      },
    )
  }
  catch (error) {
    console.error('[DMM Extractor] Extraction failed:', error)
    showNotification('DMM: ゲーム情報の抽出に失敗しました', 'error')
  }
  finally {
    isExtracting = false
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
