import type { DmmExtractedGame } from './types'
import { logger } from '@launcherg/shared'
import { extractCidFromUrl } from './utils'

const log = logger('dmm-dom-extractor')

// 抽出対象ページかどうか
export function shouldExtract(hostname: string, rootElement: HTMLElement | null): boolean {
  if (!hostname.includes('dlsoft.dmm.co.jp'))
    return false
  if (!rootElement)
    return false
  const mylibrary = document.querySelector('#mylibrary')
  const hasList = document.querySelector('ul#js-list li')
  return Boolean(mylibrary && hasList)
}

// ゲームカード候補
export function extractGameContainers(): NodeListOf<Element> {
  return document.querySelectorAll('ul#js-list > li')
}

function extractThumbnailUrl(container: Element): string | null {
  const img = container.querySelector('p.tmb span.img img') as HTMLImageElement | null
  if (!img)
    return null
  return img.getAttribute('src') || null
}

function extractTitle(container: Element): string {
  // 画像のaltの方がノイズが少ない
  const img = container.querySelector('p.tmb span.img img') as HTMLImageElement | null
  if (img && img.alt)
    return img.alt.trim()
  const txt = container.querySelector('p.tmb span.txt') as HTMLElement | null
  if (!txt)
    return ''
  // 「【割引】【還元】」等のタグを除去
  return txt.textContent?.replace(/【[^】]+】/g, '').replace(/\s+/g, ' ').trim() || ''
}

function extractCid(container: Element): string | null {
  // レビュー作成リンクに cid が含まれる
  const a = container.querySelector('.mylibraryReviewButton a[href*="cid="]') as HTMLAnchorElement | null
  const href = a?.getAttribute('href') || ''
  return extractCidFromUrl(href)
}

export function extractGameDataFromContainer(container: Element, index: number): DmmExtractedGame | null {
  try {
    const cid = extractCid(container)
    if (!cid)
      return null

    const title = extractTitle(container)
    const thumbnailUrl = extractThumbnailUrl(container) || undefined

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
  const containers = extractGameContainers()
  const games: DmmExtractedGame[] = []
  const seen = new Set<string>()
  containers.forEach((el, idx) => {
    const g = extractGameDataFromContainer(el, idx)
    if (g && !seen.has(g.store_id)) {
      seen.add(g.store_id)
      games.push(g)
    }
  })
  return games
}
