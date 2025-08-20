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

function extractGameDataFromImage(img: HTMLImageElement): DmmExtractedGame | null {
  try {
    const src = img.getAttribute('src')
    if (!src)
      return null
    const parsed = extractFromImageSrc(src)
    if (!parsed)
      return null
    const title = typeof img.alt === 'string' ? img.alt.trim() : ''
    if (!title)
      return null

    const game: DmmExtractedGame = {
      storeId: parsed.storeId,
      category: parsed.category,
      subcategory: parsed.subcategory,
      title,
      imageUrl: src,
    }
    return game
  }
  catch (e) {
    log.debug('extractGameDataFromImage error:', e)
    return null
  }
}

export function extractAllGames(): DmmExtractedGame[] {
  const scope = document.querySelector('#mylibrary') || document
  const images = Array.from(scope.querySelectorAll('img'))
  const games: DmmExtractedGame[] = []
  images.forEach((img, idx) => {
    const g = extractGameDataFromImage(img)
    if (g) {
      log.debug(`Extracted game ${idx + 1}:`, g)
      games.push(g)
    }
  })
  return games
}
