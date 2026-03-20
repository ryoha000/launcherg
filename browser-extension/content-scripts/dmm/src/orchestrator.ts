import type { DmmGame, DmmSyncGamesRequest } from '@launcherg/shared'
import type { DmmApiExtractedGame, DmmSetDetailResponse } from './api'
import type { DmmExtractedGame } from './types'
import { logger, sendExtensionRequest } from '@launcherg/shared'

import { extractDmmGamesFromSetDetailResponse } from './api'
import { getCachedPackChildrenMulti, setCachedPackChildren } from './cache'

const log = logger('dmm-orchestrator')

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

async function fetchPackDetailResponseForStoreId(storeId: string): Promise<DmmSetDetailResponse> {
  const url = new URL('/ajax/v1/library/detail/set/', window.location.origin)
  url.searchParams.set('productId', storeId)
  return await fetchJsonWithCookie<DmmSetDetailResponse>(url.toString())
}

export async function syncDmmGames(games: DmmExtractedGame[]): Promise<void> {
  if (games.length === 0)
    return
  log.info('syncDmmGames', {
    total: games.length,
    parentPackCount: games.filter(game => !!game.parentPack).length,
  })
  const dmmGames: DmmGame[] = games.map(g => ({
    id: g.storeId,
    category: g.category,
    subcategory: g.subcategory,
    title: g.title,
    imageUrl: g.imageUrl,
    parentPack: g.parentPack,
  }))
  await sendExtensionRequest({
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'syncDmmGames', value: { games: dmmGames } as DmmSyncGamesRequest },
  })
}

export async function processPacks(packGames: DmmApiExtractedGame[]): Promise<DmmExtractedGame[]> {
  const packIds = packGames.map(game => game.storeId)
  const collectedGames: DmmExtractedGame[] = []

  if (packGames.length === 0)
    return collectedGames

  log.info('processPacks', { packCount: packGames.length })
  // 事前に可能な限りキャッシュをバルク取得
  const cached = await getCachedPackChildrenMulti(packIds)

  for (const packGame of packGames) {
    const sid = packGame.storeId
    const cachedChildren = cached.get(sid)
    if (cachedChildren) {
      log.debug('pack cache hit', { storeId: sid, childCount: cachedChildren.length })
      // キャッシュヒット：ネットワークをスキップ
      collectedGames.push(...cachedChildren.map(g => ({
        ...g,
        parentPack: {
          storeId: packGame.storeId,
          category: packGame.category,
          subcategory: packGame.subcategory,
        },
      })))
      continue
    }

    // キャッシュミス：detail/set API を直接読んで childProducts を展開する
    try {
      log.debug('pack cache miss', { storeId: sid })
      const response = await fetchPackDetailResponseForStoreId(sid)
      const games = extractDmmGamesFromSetDetailResponse(response)
      log.info('pack children fetched', { storeId: sid, childCount: games.length })
      // 子リストを永続キャッシュに保存（親 pack 情報は実行時に付与する）
      await setCachedPackChildren(sid, games.map(({ parentPack: _omit, ...rest }) => rest))

      collectedGames.push(...games.map(g => ({
        ...g,
        parentPack: {
          storeId: packGame.storeId,
          category: packGame.category,
          subcategory: packGame.subcategory,
        },
      })))
    }
    catch (error) {
      log.error('pack detail fetch failed', { storeId: sid, error })
    }
    // 連続アクセスを軽減
    await new Promise(r => setTimeout(r, 500))
  }
  return collectedGames
}
