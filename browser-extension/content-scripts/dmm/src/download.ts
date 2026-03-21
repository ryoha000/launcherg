import type { DmmSetDetailResponse, DmmSingleDetailResponse } from './api'
import { logger, setDownloadIntent, showInPageNotification } from '@launcherg/shared'
import {

  extractDownloadUrlsFromSetDetail,
  extractDownloadUrlsFromSingleDetail,
} from './api'

const log = logger('dmm-download')
const DMM_DOWNLOAD_PARAM_SESSION_KEY = 'launcherg:dmm-download-param'

export interface LaunchergDownloadParam {
  type: 'download'
  value: {
    game: { storeId: string, category: string, subcategory: string }
    parentPack?: { storeId: string, category: string, subcategory: string }
  }
}

export function parseLaunchergParam(): LaunchergDownloadParam | null {
  try {
    const url = new URL(window.location.href)
    const hashParams = new URLSearchParams(url.hash.startsWith('#') ? url.hash.slice(1) : url.hash)
    const raw = url.searchParams.get('launcherg') || hashParams.get('launcherg')
    if (raw) {
      try {
        sessionStorage.setItem(DMM_DOWNLOAD_PARAM_SESSION_KEY, raw)
      }
      catch {}
    }
    const candidate = raw ?? (() => {
      try {
        return sessionStorage.getItem(DMM_DOWNLOAD_PARAM_SESSION_KEY)
      }
      catch {
        return null
      }
    })()
    if (!candidate)
      return null
    const decoded = decodeURIComponent(candidate)
    const parsed = JSON.parse(decoded) as unknown
    if (
      typeof parsed === 'object' && parsed !== null
      && (parsed as any).type === 'download'
      && typeof (parsed as any).value === 'object' && (parsed as any).value !== null
    ) {
      log.debug('launcherg param parsed', { hasParentPack: !!(parsed as LaunchergDownloadParam).value.parentPack })
      return parsed as LaunchergDownloadParam
    }
    return null
  }
  catch (e) {
    log.debug('Failed to parse launcherg param', e)
    return null
  }
}

// 現在のタブを閉じる（background 経由）
export function closeCurrentTab(): void {
  try {
    chrome.runtime.sendMessage({ type: 'close_current_tab' })
  }
  catch {}
}

function toAbsoluteDownloadUrl(path: string): string {
  return new URL(path, window.location.origin).toString()
}

async function fetchJsonWithCookie<T>(url: string): Promise<T> {
  log.debug('fetch start', url)
  const res = await fetch(url, { credentials: 'include' })
  if (!res.ok) {
    log.error('fetch failed', { url, status: res.status, statusText: res.statusText })
    throw new Error(`DMM API request failed: ${res.status} ${res.statusText}`)
  }
  log.debug('fetch ok', { url, status: res.status })
  return await res.json() as T
}

async function fetchSingleDownloadUrls(storeId: string): Promise<string[]> {
  const url = new URL('/ajax/v1/library/detail/single/', window.location.origin)
  url.searchParams.set('productId', storeId)
  log.info('fetch single download urls', { storeId, url: url.toString() })
  const payload = await fetchJsonWithCookie<DmmSingleDetailResponse>(url.toString())
  return extractDownloadUrlsFromSingleDetail(payload).map(toAbsoluteDownloadUrl)
}

async function fetchPackDownloadUrls(parentPackStoreId: string, childStoreId: string): Promise<string[]> {
  const url = new URL('/ajax/v1/library/detail/set/', window.location.origin)
  url.searchParams.set('productId', parentPackStoreId)
  log.info('fetch pack download urls', { parentPackStoreId, childStoreId, url: url.toString() })
  const payload = await fetchJsonWithCookie<DmmSetDetailResponse>(url.toString())
  return extractDownloadUrlsFromSetDetail(payload, childStoreId).map(toAbsoluteDownloadUrl)
}

interface StartDmmDownloadsResponse {
  success?: boolean
  startedCount?: number
  failedUrls?: string[]
  error?: string
}

async function startDmmDownloads(storeId: string, urls: string[]): Promise<StartDmmDownloadsResponse> {
  return await chrome.runtime.sendMessage({
    type: 'start_dmm_downloads',
    payload: { storeId, urls },
  }) as StartDmmDownloadsResponse
}

export async function initLaunchergDownloadOnceForUrl(url: string, mark: (url: string) => void, isMarked: (url: string) => boolean): Promise<void> {
  if (isMarked(url))
    return
  const p = parseLaunchergParam()
  if (!p || p.type !== 'download')
    return
  mark(url)
  log.info('Launcherg download param detected - direct download flow start')
  try {
    const downloadUrls = p.value.parentPack
      ? await fetchPackDownloadUrls(p.value.parentPack.storeId, p.value.game.storeId)
      : await fetchSingleDownloadUrls(p.value.game.storeId)
    log.info('download urls resolved', {
      storeId: p.value.game.storeId,
      parentPack: p.value.parentPack,
      count: downloadUrls.length,
    })

    await setDownloadIntent(p.value.game.storeId, {
      store: 'DMM',
      game: p.value.game,
      parentPack: p.value.parentPack,
      expected: downloadUrls.length,
      completed: 0,
      startedAt: Date.now(),
    })
    log.debug('download intent stored', { storeId: p.value.game.storeId, expected: downloadUrls.length })

    const response = await startDmmDownloads(p.value.game.storeId, downloadUrls)
    log.debug('startDmmDownloads response', response)
    if (!response?.success) {
      const message = response?.error || 'DMM: ダウンロード開始に失敗しました'
      showInPageNotification(message, 'error')
      return
    }

    try {
      sessionStorage.removeItem(DMM_DOWNLOAD_PARAM_SESSION_KEY)
    }
    catch {}
    showInPageNotification(`DMM: ダウンロードを開始しました (${response.startedCount ?? downloadUrls.length}件)`, 'success')
    closeCurrentTab()
  }
  catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    showInPageNotification(message, 'error')
  }
}
