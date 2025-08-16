import type { DmmExtractedGame } from './types'
import { logger } from '@launcherg/shared'

const log = logger('dmm-dom-extractor')

// 抽出対象ページかどうか
export function shouldExtract(hostname: string, rootElement: HTMLElement | null): boolean {
  if (!hostname.includes('dlsoft.dmm.co.jp'))
    return false
  if (!rootElement)
    return false
  return true
}

function extractFromImageSrc(src: string | null): { storeId: string, category: string, subcategory: string } | null {
  if (!src)
    return null
  const url = new URL(src)
  if (url.hostname !== 'pics.dmm.co.jp')
    return null
  const pathname = url.pathname
  const parts = pathname.split('/')
  // ['', 'digital', 'pcgame', '{storeId}', '{filename}'] という構造を想定
  if (parts.length < 4)
    return null
  const category = parts[1]
  const subcategory = parts[2]
  const storeId = parts[3]
  return { storeId, category, subcategory }
}

function findPreferredImage(container: Element): HTMLImageElement | null {
  if (container instanceof HTMLImageElement)
    return extractFromImageSrc(container.getAttribute('src')) ? container : null
  const nodeList = container.querySelectorAll('img') as NodeListOf<HTMLImageElement>
  if (!nodeList || nodeList.length === 0)
    return null
  // srcが対応フォーマットのものを候補に限定
  const candidates: HTMLImageElement[] = []
  nodeList.forEach((img) => {
    if (extractFromImageSrc(img.getAttribute('src')))
      candidates.push(img)
  })
  if (candidates.length === 0)
    return null
  // alt が非空のものを優先し、なければ最初の候補
  const withAlt = candidates.find(img => typeof img.alt === 'string' && img.alt.trim().length > 0)
  return withAlt || candidates[0]
}

function extractThumbnailUrl(container: Element): string | null {
  const img = findPreferredImage(container)
  return img?.getAttribute('src') || null
}

function extractTitle(container: Element): string {
  const img = findPreferredImage(container)
  if (img && typeof img.alt === 'string')
    return img.alt.trim()
  return ''
}

// deriveStoreIdFromImageSrc は extractFromImageSrc に置き換え

export function extractGameDataFromContainer(container: Element, index: number): DmmExtractedGame | null {
  try {
    const thumbnailUrl = extractThumbnailUrl(container) || undefined
    const parsed = extractFromImageSrc(thumbnailUrl || null)
    const cid = parsed?.storeId || undefined
    const title = extractTitle(container)

    if (!thumbnailUrl || !cid)
      return null

    const game: DmmExtractedGame = {
      store_id: cid,
      title,
      purchase_url: `https://dlsoft.dmm.co.jp/mylibrary/?cid=${cid}`,
      purchase_date: '',
      thumbnail_url: thumbnailUrl,
      additional_data: {},
    }

    log.debug(`Extracted game ${index + 1}:`, game)
    return game
  }
  catch (e) {
    log.debug('extractGameDataFromContainer error:', e)
    return null
  }
}

export function extractAllGames(): DmmExtractedGame[] {
  const scope = document.querySelector('#mylibrary') || document
  const images = Array.from(scope.querySelectorAll('img'))
  const games: DmmExtractedGame[] = []
  const seen = new Set<string>()
  images.forEach((img, idx) => {
    if (!extractFromImageSrc(img.getAttribute('src')))
      return
    const li = (img as Element).closest('li') as Element | null
    const container = li || (img.parentElement as Element | null)
    const hasReviewButton = container?.querySelector('.mylibraryReviewButton a')
    if (!hasReviewButton)
      return
    const g = extractGameDataFromContainer(container || img, idx)
    if (g && !seen.has(g.store_id)) {
      seen.add(g.store_id)
      games.push(g)
    }
  })
  return games
}
