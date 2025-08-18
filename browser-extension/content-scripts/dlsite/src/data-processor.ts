// データ処理関連の純粋関数

import type { DlsiteExtractedGame } from './types'

// DLsiteの日付フォーマットを正規化する純粋関数
export function normalizeDLsiteDate(dateStr: string): string {
  try {
    // DLsite日付フォーマット対応: "YYYY年MM月DD日", "YYYY/MM/DD", "YYYY-MM-DD"
    // 日本時間（JST）基準での正規化: 文字列から年月日を直接抽出し、タイムゾーンに依存しない整形を行う
    const normalized = dateStr
      .replace(/年/g, '/')
      .replace(/月/g, '/')
      .replace(/日/g, '')
      .replace(/\s+/g, '')
      .replace(/-/g, '/')

    const match = normalized.match(/^(\d{4})\/(\d{1,2})\/(\d{1,2})$/)
    if (!match)
      return dateStr

    const year = Number(match[1])
    const month = Number(match[2])
    const day = Number(match[3])
    if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day))
      return dateStr

    const mm = String(month).padStart(2, '0')
    const dd = String(day).padStart(2, '0')
    return `${year}-${mm}-${dd}`
  }
  catch {
    return dateStr
  }
}

// DLsiteのタイトルをクリーンアップする純粋関数
export function cleanDLsiteTitle(title: string): string {
  // DLsiteのタイトルから不要な情報を除去
  return title
    .replace(/\[.*?\]/g, '') // [サークル名] などを除去
    .replace(/（.*?）/g, '') // 全角括弧の内容を除去
    .replace(/\(.*?\)/g, '') // 半角括弧の内容を除去
    .replace(/\s+/g, ' ') // 連続する空白を単一の空白に
    .trim()
}

// URLを正規化する純粋関数
export function normalizeUrl(url: string, _urlType: 'purchase' | 'image'): string {
  if (!url)
    return url

  // HTTPSプロトコルの追加
  if (url.startsWith('//')) {
    return `https:${url}`
  }

  if (!url.startsWith('http')) {
    return `https://play.dlsite.com${url}`
  }

  return url
}

// store_idを正規化する純粋関数
export function normalizeStoreId(storeId: string, purchaseUrl: string): string {
  if (!storeId)
    return storeId

  // 既に正しい形式ならそれを優先
  if (/^(?:RJ|VJ|BJ)\d+$/.test(storeId))
    return storeId

  // URLから作品コードを抽出
  const match = purchaseUrl.match(/\/(RJ|VJ|BJ)(\d+)/)
  if (match)
    return match[1] + match[2]

  // 数字のみの場合はRJを付加
  if (/^\d+$/.test(storeId))
    return `RJ${storeId}`

  return storeId
}

// 作品の種類を判定する純粋関数
export function determineWorkType(storeId: string): string {
  if (storeId.startsWith('RJ')) {
    return 'doujin'
  }
  else if (storeId.startsWith('VJ')) {
    return 'voice'
  }
  else if (storeId.startsWith('BJ')) {
    return 'book'
  }
  return 'unknown'
}

// DLsiteのゲームデータを処理する純粋関数
export function processDLsiteGame(game: DlsiteExtractedGame): DlsiteExtractedGame {
  const processedGame: DlsiteExtractedGame = {
    ...game,
    title: game.title ? cleanDLsiteTitle(game.title) : '',
    imageUrl: normalizeUrl(game.imageUrl, 'image'),
  }
  return processedGame
}

// ゲームデータのリストを処理する純粋関数
export function processGames(games: DlsiteExtractedGame[]): DlsiteExtractedGame[] {
  return games.map(game => processDLsiteGame(game))
}
