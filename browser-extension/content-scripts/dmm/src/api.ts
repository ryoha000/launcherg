import type { DmmExtractedGame } from './types'
import { logger, normalizeTitle } from '@launcherg/shared'

const log = logger('dmm-api')

export const DMM_HOOK_MESSAGE_SOURCE = 'launcherg'
export const DMM_LIBRARY_MESSAGE_TYPE = 'launcherg:dmm-library-response'
export const DMM_LIBRARY_SCRIPT_ID = 'launcherg-dmm-network-hook'
export const DMM_LIBRARY_SCRIPT_PATH = 'content-scripts/dmm-network-hook.js'

const DMM_LIBRARY_HOST = 'dlsoft.dmm.co.jp'
const DMM_LIBRARY_PATH = '/ajax/v1/library'

export interface DmmLibraryItem {
  contentId?: string
  productId?: string
  libraryProductType?: string
  floor?: string
  title?: string
  packageImageUrl?: string
}

export interface DmmLibraryResponse {
  error: string | null
  body?: {
    totalCount?: number
    library?: DmmLibraryItem[]
  } | null
}

export interface DmmLibraryHookMessageData {
  source: typeof DMM_HOOK_MESSAGE_SOURCE
  type: typeof DMM_LIBRARY_MESSAGE_TYPE
  pageUrl: string
  requestUrl: string
  payload: DmmLibraryResponse
}

export interface DmmApiExtractedGame extends DmmExtractedGame {
  isPack: boolean
}

export function isDmmLibraryApiUrl(url: string): boolean {
  try {
    const parsed = new URL(url, `https://${DMM_LIBRARY_HOST}`)
    return parsed.hostname === DMM_LIBRARY_HOST && parsed.pathname === DMM_LIBRARY_PATH
  }
  catch {
    return false
  }
}

export function isDmmLibraryResponse(value: unknown): value is DmmLibraryResponse {
  if (typeof value !== 'object' || value === null)
    return false
  return 'error' in value
}

function parseImageMeta(imageUrl: string | undefined, fallbackStoreId: string): Pick<DmmExtractedGame, 'storeId' | 'category' | 'subcategory' | 'imageUrl'> | null {
  if (!imageUrl)
    return null

  try {
    const parsed = new URL(imageUrl)
    const parts = parsed.pathname.split('/')
    if (parts.length >= 5) {
      return {
        storeId: parts[3] || fallbackStoreId,
        category: parts[1] || 'digital',
        subcategory: parts[2] || 'pcgame',
        imageUrl: parsed.toString(),
      }
    }
  }
  catch {}

  return null
}

function parseFloorMeta(floor: string | undefined, fallbackStoreId: string, imageUrl: string): Pick<DmmExtractedGame, 'storeId' | 'category' | 'subcategory' | 'imageUrl'> | null {
  const normalized = (floor || '').toLowerCase()
  if (!normalized)
    return null

  const category = normalized.includes('mono') ? 'mono' : 'digital'
  if (normalized.includes('doujin')) {
    return {
      storeId: fallbackStoreId,
      category,
      subcategory: 'doujin',
      imageUrl,
    }
  }
  if (normalized.includes('pcgame')) {
    return {
      storeId: fallbackStoreId,
      category,
      subcategory: 'pcgame',
      imageUrl,
    }
  }
  return null
}

export function convertDmmLibraryItem(item: DmmLibraryItem): DmmApiExtractedGame | null {
  const fallbackStoreId = item.productId || item.contentId || ''
  const title = normalizeTitle(item.title || '')
  if (!fallbackStoreId || !title)
    return null

  const parsed
    = parseImageMeta(item.packageImageUrl, fallbackStoreId)
      || parseFloorMeta(item.floor, fallbackStoreId, item.packageImageUrl || '')

  if (!parsed) {
    log.warn('Unable to parse DMM library item:', item)
    return null
  }

  return {
    ...parsed,
    title,
    isPack: item.libraryProductType === 'set',
  }
}

export function extractDmmGamesFromApiResponse(response: DmmLibraryResponse): DmmApiExtractedGame[] {
  const library = response.body?.library
  if (!Array.isArray(library))
    return []

  return library
    .map(convertDmmLibraryItem)
    .filter((game): game is DmmApiExtractedGame => game !== null)
}

export function splitDmmApiGames(games: DmmApiExtractedGame[]): {
  normalGames: DmmExtractedGame[]
  packGames: DmmApiExtractedGame[]
} {
  const normalGames: DmmExtractedGame[] = []
  const packGames: DmmApiExtractedGame[] = []

  for (const game of games) {
    if (game.isPack) {
      packGames.push(game)
      continue
    }
    const { isPack: _omit, ...normalGame } = game
    normalGames.push(normalGame)
  }

  return { normalGames, packGames }
}

export function buildDmmPayloadKey(message: DmmLibraryHookMessageData): string {
  const ids = (message.payload.body?.library || [])
    .map(item => `${item.productId || item.contentId || ''}:${item.libraryProductType || ''}`)
    .join('|')
  return `${message.requestUrl}::${ids}`
}
