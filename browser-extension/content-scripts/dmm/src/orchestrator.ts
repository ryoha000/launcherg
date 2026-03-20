import type { DmmGame, DmmSyncGamesRequest, ExtensionRequest } from '@launcherg/shared'
import type { DmmExtractedGame } from './types'

import { sendExtensionRequest } from '@launcherg/shared'
import { getCachedPackChildrenMulti, setCachedPackChildren } from './cache'
import { fetchPackDetailHtmlForItemId, findDetailItemIdForStoreId } from './pack-helpers'
import { parsePackModal } from './pack-parser'

export async function fetchPackParentMap(): Promise<Map<string, number>> {
  const req: ExtensionRequest = {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'getDmmOmitWorks', value: {} },
  }
  const res = await sendExtensionRequest(req)
  if (res?.response?.case !== 'getDmmOmitWorksResult')
    throw new Error('Unexpected response from getDmmOmitWorks')
  const payload = res.response.value
  const map = new Map<string, number>()
  for (const it of payload.items) {
    map.set(it.dmm.storeId, it.workId)
  }
  return map
}

export async function syncDmmGames(games: DmmExtractedGame[]): Promise<void> {
  if (games.length === 0)
    return
  const dmmGames: DmmGame[] = games.map(g => ({
    id: g.storeId,
    category: g.category,
    subcategory: g.subcategory,
    title: g.title,
    imageUrl: g.imageUrl,
    parentPackWorkId: g.parentPackWorkId,
  }))
  const request: ExtensionRequest = {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'syncDmmGames', value: { games: dmmGames } as DmmSyncGamesRequest },
  }
  await sendExtensionRequest(request)
}

export async function processPacks(packSet: Set<string>, parentMap?: Map<string, number>): Promise<DmmExtractedGame[]> {
  const packIds = Array.from(packSet)
  const collectedGames: DmmExtractedGame[] = []

  // 事前に可能な限りキャッシュをバルク取得
  const cached = await getCachedPackChildrenMulti(packIds)

  for (const sid of packIds) {
    const parentId = parentMap?.get(sid)
    const cachedChildren = cached.get(sid)
    if (cachedChildren) {
      // キャッシュヒット：ネットワークをスキップ
      if (parentId) {
        collectedGames.push(...cachedChildren.map(g => ({ ...g, parentPackWorkId: parentId })))
      }
      else {
        collectedGames.push(...cachedChildren)
      }
      continue
    }

    // キャッシュミス：DOM から detail item を探し、fetch→parse
    const itemId = findDetailItemIdForStoreId(sid)
    if (!itemId)
      continue
    try {
      const html = await fetchPackDetailHtmlForItemId(itemId, 12000)
      const games = parsePackModal(html)
      // 子リストを永続キャッシュに保存（parentPackWorkId は保存しない）
      await setCachedPackChildren(sid, games.map(({ parentPackWorkId: _omit, ...rest }) => rest))

      if (parentId) {
        collectedGames.push(...games.map(g => ({ ...g, parentPackWorkId: parentId })))
      }
      else {
        collectedGames.push(...games)
      }
    }
    catch {}
    // 連続アクセスを軽減
    await new Promise(r => setTimeout(r, 500))
  }
  return collectedGames
}
