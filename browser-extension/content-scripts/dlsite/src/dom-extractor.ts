// DOM操作関連の純粋関数

import type { ExtractedGameData } from '@launcherg/shared'
import { logger } from '@launcherg/shared'
import { extractStoreIdFromUrl } from './utils'

const log = logger('dlsite-extractor')

// ページが抽出対象かどうかを判定する純粋関数
export function shouldExtract(hostname: string, rootElement: HTMLElement | null): boolean {
  // ページURL確認
  if (!hostname.includes('dlsite.com')) {
    return false
  }

  if (!rootElement) return false

  // 旧実装セレクタ or 安定痕跡（img.dlsite.jp を含む画像/背景画像）
  const hasOldSelectors =
    document.querySelector('._thumbnail_1kd4u_117') !== null
    || document.querySelector('[data-index]') !== null

  const hasDlsiteImages =
    document.querySelector('img[src*="img.dlsite.jp"]') !== null
    || document.querySelector('[style*="img.dlsite.jp"]') !== null

  return hasOldSelectors || hasDlsiteImages
}

// ゲームコンテナー要素を取得する純粋関数
export function extractGameContainers(): NodeListOf<Element> {
  return document.querySelectorAll('[data-index]')
}

function findThumbnailUrlInContainer(container: Element): string | null {
  // 優先: <img src> / srcset
  const img = container.querySelector('img') as HTMLImageElement | null
  if (img) {
    const src = img.src || ''
    if (/img\.dlsite\.jp/.test(src)) return src
    const srcset = img.srcset || ''
    const candidates = srcset.split(',').map(s => s.trim().split(' ')[0]).filter(Boolean)
    const best = candidates.find(u => /img\.dlsite\.jp/.test(u))
    if (best) return best
  }
  // 次: 背景画像
  const bgHolder = container.querySelector('[style*="background-image"]') as HTMLElement | null
  if (bgHolder) {
    const bg = bgHolder.style.backgroundImage || ''
    const m = bg.match(/url\("?(.+?)"?\)/)
    if (m && /img\.dlsite\.jp/.test(m[1])) return m[1]
  }
  return null
}

function extractTitleHeuristically(container: Element): string {
  const heading = container.querySelector('[role="heading"], h1, h2, h3, [title]') as HTMLElement | null
  if (heading) {
    const t = (heading.getAttribute('title') || heading.textContent || '').trim()
    if (t) return t
  }
  const spans = container.querySelectorAll('span, div')
  for (const el of Array.from(spans)) {
    const t = (el.textContent || '').trim()
    if (t && t.length >= 2 && !/購入/.test(t) && !/\d{4}[-/年]\d{1,2}[-/月]\d{1,2}/.test(t)) return t
  }
  return ''
}

function extractMakerHeuristically(container: Element): string {
  const a = container.querySelector('a[href*="/circle/"]') as HTMLAnchorElement | null
  if (a?.textContent) return a.textContent.trim()
  const spans = container.querySelectorAll('span, div')
  for (const el of Array.from(spans)) {
    const t = (el.textContent || '').trim()
    if (t && t.length <= 40 && !/\d{4}[-/年]\d{1,2}[-/月]\d{1,2}/.test(t)) return t
  }
  return ''
}

function extractDateHeuristically(container: Element): string {
  const text = container.textContent || ''
  const m = text.match(/(\d{4})[-/年](\d{1,2})[-/月](\d{1,2})/)
  if (!m) return ''
  const y = m[1]
  const mm = String(m[2]).padStart(2, '0')
  const dd = String(m[3]).padStart(2, '0')
  return `${y}-${mm}-${dd}`
}

// コンテナー要素からゲームデータを抽出する純粋関数
export function extractGameDataFromContainer(
  container: Element,
  index: number,
): ExtractedGameData | null {
  try {
    // まずは旧DOM構造のサムネイル
    let thumbnailUrl: string | null = null
    const oldThumb = container.querySelector('._thumbnail_1kd4u_117 span') as HTMLElement | null
    if (oldThumb) {
      const bgImage = oldThumb.style.backgroundImage
      const thumbnailMatch = bgImage.match(/url\("?(.+?)"\)/)
      if (thumbnailMatch) thumbnailUrl = thumbnailMatch[1]
    }
    // フォールバック: img/srcset or 背景画像
    if (!thumbnailUrl) {
      thumbnailUrl = findThumbnailUrlInContainer(container)
    }
    if (!thumbnailUrl) return null

    // URLからstore_idを抽出
    const storeId = extractStoreIdFromUrl(thumbnailUrl)
    log.debug(`Extracted store_id "${storeId}" from URL: ${thumbnailUrl}`)
    if (!storeId) {
      return null
    }

    // タイトルを抽出
    const titleElement = container.querySelector('._workName_1kd4u_192 span')
    const title = (titleElement?.textContent?.trim() || extractTitleHeuristically(container))

    // メーカー名を抽出
    const makerElement = container.querySelector('._makerName_1kd4u_196 span')
    const makerName = (makerElement?.textContent?.trim() || extractMakerHeuristically(container))

    // 購入日を抽出（親要素から探す）
    let purchaseDate = ''
    const headerElement = container.closest('[data-index]')?.querySelector('._header_1kd4u_27 span')
    if (headerElement?.textContent?.includes('購入')) {
      purchaseDate = headerElement.textContent.replace('購入', '').trim()
    }
    if (!purchaseDate) {
      purchaseDate = extractDateHeuristically(container)
    }

    // 購入URLを構築
    const purchaseUrl = `https://play.dlsite.com/maniax/work/=/product_id/${storeId}.html`

    const gameData: ExtractedGameData = {
      store_id: storeId,
      title,
      purchase_url: purchaseUrl,
      purchase_date: purchaseDate,
      thumbnail_url: thumbnailUrl,
      additional_data: {
        maker_name: makerName,
      },
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
export function extractAllGames(): ExtractedGameData[] {
  const gameContainers = extractGameContainers()
  log.debug(`Found ${gameContainers.length} potential game containers`)

  const games: ExtractedGameData[] = []
  const seenStoreIds = new Set<string>()

  gameContainers.forEach((container, index) => {
    const gameData = extractGameDataFromContainer(container, index)

    if (gameData) {
      // 重複チェック
      if (!seenStoreIds.has(gameData.store_id)) {
        seenStoreIds.add(gameData.store_id)
        games.push(gameData)
      }
    }
  })

  return games
}
