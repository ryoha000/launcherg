import type { DlsiteExtractedGame } from './types'
import { logger, normalizeTitle } from '@launcherg/shared'

export const DLSITE_HOOK_MESSAGE_SOURCE = 'launcherg'
export const DLSITE_WORKS_MESSAGE_TYPE = 'launcherg:dlsite-works-response'
export const DLSITE_WORKS_SCRIPT_ID = 'launcherg-dlsite-network-hook'
export const DLSITE_WORKS_SCRIPT_PATH = 'content-scripts/dlsite-network-hook.js'

const DLSITE_WORKS_HOST = 'play.dlsite.com'
const DLSITE_WORKS_PATH = '/api/v3/content/works'
const log = logger('dlsite-api')

export interface DlsiteWorkItem {
  workno?: string
  site_id?: string
  file_type?: string
  name?: {
    ja_JP?: string
  }
  work_files?: {
    main?: string
  }
}

export interface DlsiteWorksResponse {
  works?: DlsiteWorkItem[]
}

export interface DlsiteWorksHookMessageData {
  source: typeof DLSITE_HOOK_MESSAGE_SOURCE
  type: typeof DLSITE_WORKS_MESSAGE_TYPE
  pageUrl: string
  requestUrl: string
  payload: DlsiteWorksResponse
}

export function isDlsiteWorksApiUrl(url: string): boolean {
  try {
    const parsed = new URL(url, `https://${DLSITE_WORKS_HOST}`)
    return parsed.hostname === DLSITE_WORKS_HOST && parsed.pathname === DLSITE_WORKS_PATH
  }
  catch {
    return false
  }
}

export function isDlsiteWorksResponse(value: unknown): value is DlsiteWorksResponse {
  return typeof value === 'object' && value !== null && 'works' in value
}

function normalizeDlsiteCategory(siteId: string | undefined): string | null {
  if (!siteId)
    return null
  if (siteId === 'maniax' || siteId === 'pro')
    return siteId
  return siteId
}

function normalizeDlsiteFileType(fileType: string | undefined): string {
  const normalized = fileType?.trim().toUpperCase()
  return normalized || 'unknown'
}

export function convertDlsiteWorkItem(work: DlsiteWorkItem): DlsiteExtractedGame | null {
  if (normalizeDlsiteFileType(work.file_type) !== 'EXE')
    return null

  const storeId = work.workno || ''
  const category = normalizeDlsiteCategory(work.site_id)
  const title = normalizeTitle(work.name?.ja_JP || '')
  const imageUrl = work.work_files?.main || ''

  if (!storeId || !category || !title || !imageUrl)
    return null

  return {
    storeId,
    category,
    title,
    imageUrl,
  }
}

export function extractDlsiteGamesFromApiResponse(response: DlsiteWorksResponse): DlsiteExtractedGame[] {
  const works = response.works
  if (!Array.isArray(works))
    return []

  const skippedFileTypes = new Set<string>()
  const games = works.flatMap((work) => {
    const normalizedFileType = normalizeDlsiteFileType(work.file_type)
    if (normalizedFileType !== 'EXE') {
      skippedFileTypes.add(normalizedFileType)
      return []
    }

    const game = convertDlsiteWorkItem(work)
    return game ? [game] : []
  })

  if (skippedFileTypes.size > 0) {
    log.debug('DLsite: EXE 以外の作品をスキップしました', {
      fileTypes: Array.from(skippedFileTypes).sort(),
    })
  }

  return games
}

export function buildDlsitePayloadKey(message: DlsiteWorksHookMessageData): string {
  const ids = (message.payload.works || [])
    .map(work => `${work.workno || ''}:${work.site_id || ''}`)
    .join('|')
  return `${message.requestUrl}::${ids}`
}
