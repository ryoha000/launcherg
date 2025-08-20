// DOM操作関連の純粋関数

import type { DlsiteExtractedGame } from './types'
import { logger } from '@launcherg/shared'
import { extractStoreIdFromUrl } from './utils'

const log = logger('dlsite-extractor')

// カテゴリラベルの厳密一致 → 種別マッピング（共有定義）
const STRICT_LABEL_TO_KIND_ENTRIES: ReadonlyArray<readonly [
  string,
  'game' | 'manga_cg' | 'video' | 'audio' | 'book',
]> = [
  ['ゲーム', 'game'],
  ['マンガ・CG', 'manga_cg'],
  ['動画', 'video'],
  ['音声', 'audio'],
  ['書籍', 'book'],
  ['コミック', 'book'],
] as const
const STRICT_LABEL_TO_KIND = new Map<string, 'game' | 'manga_cg' | 'video' | 'audio' | 'book'>(
  STRICT_LABEL_TO_KIND_ENTRIES,
)

// ページが抽出対象かどうかを判定する純粋関数
export function shouldExtract(hostname: string, rootElement: HTMLElement | null): boolean {
  // ページURL確認
  if (!hostname.includes('dlsite.com')) {
    return false
  }

  if (!rootElement)
    return false

  // 安定痕跡（img.dlsite.jp を含む画像/背景画像）
  const hasDlsiteImages
    = document.querySelector('img[src*="img.dlsite.jp"]') !== null
      || document.querySelector('[style*="img.dlsite.jp"]') !== null

  return hasDlsiteImages
}

// ゲームコンテナー要素を取得する純粋関数
export function extractGameContainers(): NodeListOf<Element> {
  // サムネイル画像そのもの、または背景画像を持つ要素を候補にする
  return document.querySelectorAll('img, [style*="img.dlsite.jp"], [style*="background-image"]')
}

function findImageUrlInContainer(container: Element): string | null {
  // 優先: <img src> / srcset
  const imgSelf = (container as HTMLImageElement).src !== undefined ? (container as HTMLImageElement) : null
  const img = imgSelf || (container.querySelector('img') as HTMLImageElement | null)
  if (img) {
    const src = img.src || ''
    if (src)
      return src
    const srcset = img.srcset || ''
    const candidates = srcset.split(',').map(s => s.trim().split(' ')[0]).filter(Boolean)
    const best = candidates.find(u => u)
    if (best)
      return best
  }
  // 次: 背景画像
  const selfStyleEl = (container as HTMLElement).style?.backgroundImage ? (container as HTMLElement) : null
  const bgHolder = selfStyleEl || (container.querySelector('[style*="background-image"]') as HTMLElement | null)
  if (bgHolder) {
    const bg = bgHolder.style.backgroundImage || ''
    const m = bg.match(/url\("?(.+?)"?\)/)
    if (m)
      return m[1]
  }
  return null
}

// 葉ノード抽出のためのヘルパー
function isDateLike(text: string): boolean {
  return /\d{4}年\d{1,2}月\d{1,2}日/.test(text) || /\d{4}[-/]\d{1,2}[-/]\d{1,2}/.test(text)
}

function isCategoryWord(text: string): boolean {
  return STRICT_LABEL_TO_KIND.has(text)
}

function collectLeafTexts(container: Element): Array<{ el: Element, text: string }> {
  const results: Array<{ el: Element, text: string }> = []
  const all = container.querySelectorAll('*')
  for (const el of Array.from(all)) {
    const elem = el as Element
    if (elem.children.length === 0) {
      const t = (elem.textContent || '').trim()
      if (t)
        results.push({ el: elem, text: t })
    }
  }
  return results
}

function extractTitle(container: Element): string {
  const leafs = collectLeafTexts(container)
  for (const { text } of leafs) {
    if (/購入/.test(text))
      continue
    if (isDateLike(text))
      continue
    if (isCategoryWord(text))
      continue
    return text
  }
  return ''
}

// カード内のラベルから作品種別を推定（ゲーム・マンガ/CG・動画・音声・書籍）。見つからなければnull
function detectWorkLabelInCard(card: Element): 'game' | 'manga_cg' | 'video' | 'audio' | 'book' | null {
  const leafs = collectLeafTexts(card)
  // 厳密一致を優先（共有定義）
  for (const { text } of leafs) {
    const mapped = STRICT_LABEL_TO_KIND.get(text)
    if (mapped)
      return mapped
  }
  // バリエーション（例えば「マンガ」「CG」が分割されている等）を緩く検出
  const joinedTexts = leafs.map(l => l.text).join(' ')
  if (/ゲーム/.test(joinedTexts))
    return 'game'
  if (/(?:マンガ|漫画).*CG|CG.*(?:マンガ|漫画)/.test(joinedTexts))
    return 'manga_cg'
  if (/動画/.test(joinedTexts))
    return 'video'
  if (/音声/.test(joinedTexts))
    return 'audio'
  if (/書籍|コミック|電子書籍/.test(joinedTexts))
    return 'book'
  return null
}

// コンテナー要素からゲームデータを抽出する純粋関数
export function extractGameDataFromContainer(
  container: Element,
  index: number,
): DlsiteExtractedGame | null {
  try {
    // 画像要素からカードコンテキストを特定
    const card
      = container.closest('[data-index], article, li, [role="listitem"]')
        || container.parentElement
        || container

    // 画像URL検出: img/srcset or 背景画像（カード全体から）
    const imageUrl: string | null = findImageUrlInContainer(card)
    if (!imageUrl)
      return null

    // URLからstoreIdを抽出
    const storeId = extractStoreIdFromUrl(imageUrl)
    log.debug(`Extracted storeId "${storeId}" from URL: ${imageUrl}`)
    if (!storeId) {
      return null
    }

    // カード内の種別ラベルを確認し、ゲーム以外（マンガ・CG/動画/音声/書籍）は弾く
    const detectedLabel = detectWorkLabelInCard(card)
    if (detectedLabel && detectedLabel !== 'game') {
      log.debug(`Skip non-game item (label=${detectedLabel}) for storeId=${storeId}`)
      return null
    }

    // タイトルを抽出
    let title = extractTitle(card)

    // カテゴリーを画像URLから推定
    // .../images2/work/(professional|doujin)/...
    const categoryPathMatch = imageUrl.match(/\/images\d\/work\/(professional|doujin)\//)
    const rawCategory = categoryPathMatch ? categoryPathMatch[1] : 'doujin'
    const category = rawCategory === 'professional' ? 'pro' : 'maniax'

    const gameData: DlsiteExtractedGame = {
      storeId,
      category,
      title,
      imageUrl,
    }

    log.debug(`Extracted game ${index + 1}:`, gameData)

    return gameData
  }
  catch (error) {
    log.debug(`Error extracting game from container ${index}:`, error)
    return null
  }
}

// すべてのゲームデータを抽出する純粋関数
export function extractAllGames(): DlsiteExtractedGame[] {
  const gameContainers = extractGameContainers()
  log.debug(`Found ${gameContainers.length} potential game containers`)

  const games: DlsiteExtractedGame[] = []
  const seenStoreIds = new Set<string>()

  gameContainers.forEach((container, index) => {
    const gameData = extractGameDataFromContainer(container, index)

    if (gameData) {
      // 重複チェック
      if (!seenStoreIds.has(gameData.storeId)) {
        seenStoreIds.add(gameData.storeId)
        games.push(gameData)
      }
    }
  })

  return games
}
