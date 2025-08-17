// DOM操作関連の純粋関数

import type { DlsiteExtractedGame } from './types'
import { logger } from '@launcherg/shared'
import { extractStoreIdFromUrl } from './utils'

const log = logger('dlsite-extractor')

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

function findThumbnailUrlInContainer(container: Element): string | null {
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
  return text === 'ゲーム' || text === '音声' || text === '書籍'
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

    // サムネイルURL検出: img/srcset or 背景画像（カード全体から）
    const thumbnailUrl: string | null = findThumbnailUrlInContainer(card)
    if (!thumbnailUrl)
      return null

    // URLからstoreIdを抽出
    const storeId = extractStoreIdFromUrl(thumbnailUrl)
    log.debug(`Extracted storeId "${storeId}" from URL: ${thumbnailUrl}`)
    if (!storeId) {
      return null
    }

    // タイトルを抽出
    let title = extractTitle(card)

    // カテゴリーをサムネイルURLから推定
    // .../images2/work/(professional|doujin)/...
    const categoryPathMatch = thumbnailUrl.match(/\/images\d\/work\/(professional|doujin)\//)
    const rawCategory = categoryPathMatch ? categoryPathMatch[1] : 'doujin'
    const category = rawCategory === 'professional' ? 'pro' : 'maniax'

    const gameData: DlsiteExtractedGame = {
      storeId,
      category,
      title,
      thumbnailUrl,
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
