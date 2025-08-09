import type { DmmExtractedGame } from './types'

export function normalizeDmmDate(dateStr: string): string {
  try {
    const normalized = dateStr
      .replace(/年/g, '/')
      .replace(/月/g, '/')
      .replace(/日/g, '')
      .replace(/-/g, '/')
      .replace(/\s+/g, '')

    const m = normalized.match(/^(\d{4})\/(\d{1,2})\/(\d{1,2})$/)
    if (!m)
      return dateStr
    const y = Number(m[1])
    const mo = String(Number(m[2])).padStart(2, '0')
    const d = String(Number(m[3])).padStart(2, '0')
    return `${y}-${mo}-${d}`
  }
  catch {
    return dateStr
  }
}

export function normalizeUrl(url: string, base: 'purchase' | 'thumbnail'): string {
  if (!url)
    return url
  if (url.startsWith('//'))
    return `https:${url}`
  if (!url.startsWith('http')) {
    return base === 'purchase' ? `https://dlsoft.dmm.co.jp${url}` : `https://dlsoft.dmm.co.jp${url}`
  }
  return url
}

export function processDmmGame(game: DmmExtractedGame): DmmExtractedGame {
  const processed: DmmExtractedGame = {
    ...game,
    additional_data: { ...game.additional_data },
  }

  processed.purchase_url = normalizeUrl(processed.purchase_url, 'purchase')
  if (processed.thumbnail_url)
    processed.thumbnail_url = normalizeUrl(processed.thumbnail_url, 'thumbnail')

  if (processed.purchase_date)
    processed.purchase_date = normalizeDmmDate(processed.purchase_date)

  processed.additional_data.store_name = 'DMM'
  processed.additional_data.extraction_source = 'dmm-dom-extractor'
  processed.additional_data.extraction_timestamp = new Date().toISOString()

  return processed
}

export function processGames(games: DmmExtractedGame[]): DmmExtractedGame[] {
  return games.map(processDmmGame)
}
