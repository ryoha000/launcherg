// Extension Internal protobuf types

import type { ExtractedGameData } from '@launcherg/shared'
import { create, fromJson, toJson } from '@bufbuild/protobuf'
import {
  addNotificationStyles,
  generateRequestId,
  sendSyncRequest,
  showNotification,
  waitForPageLoad,
} from '@launcherg/shared'

import {
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GetConfigRequestSchema,
} from '@launcherg/shared/proto/extension_internal'

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

    // 共通の同期機能を使用
    sendSyncRequest(
      'DMM',
      processedGames,
      'dmm-extractor',
      (response) => {
        console.log('[DMM Extractor] Sync successful:', response)
        showNotification(
          `DMM: ${processedGames.length}個のゲームを同期しました`,
        )
      },
      (error) => {
        console.error('[DMM Extractor] Sync failed:', error)
        showNotification('DMM: 同期に失敗しました', 'error')
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

// 通知スタイルを追加
addNotificationStyles()

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
