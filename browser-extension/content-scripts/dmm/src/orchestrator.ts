import type { DmmGame, DmmSyncGamesRequest, ExtensionRequest } from '@launcherg/shared'
import type { DmmExtractedGame } from './types'

import { sendExtensionRequest } from '@launcherg/shared'
import { fetchPackDetailHtmlForItemId, findDetailItemIdForStoreId } from './pack-helpers'
import { parsePackModal } from './pack-parser'

export async function fetchPackIds(): Promise<Set<string>> {
  const packReq: ExtensionRequest = {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'getDmmOmitWorks', value: {} },
  }
  const packResJson = await sendExtensionRequest(packReq)
  const dmmPackIds: string[] = []
  if (packResJson?.response?.case === 'getDmmOmitWorksResult') {
    const payload = packResJson.response.value
    dmmPackIds.push(...payload.items.map(it => it.dmm.storeId))
  }
  else {
    throw new Error('Unexpected response from getDmmOmitWorks')
  }
  return new Set<string>(dmmPackIds)
}

export async function fetchPackParentMap(): Promise<Map<string, number>> {
  const packReq: ExtensionRequest = {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'getDmmOmitWorks', value: {} },
  }
  const packResJson = await sendExtensionRequest(packReq)
  const m = new Map<string, number>()
  if (packResJson?.response?.case === 'getDmmOmitWorksResult') {
    const payload = packResJson.response.value
    for (const it of payload.items) {
      m.set(it.dmm.storeId, it.workId)
    }
  }
  else {
    throw new Error('Unexpected response from getDmmOmitWorks')
  }
  return m
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
  for (const sid of packIds) {
    const itemId = findDetailItemIdForStoreId(sid)
    if (!itemId)
      continue
    try {
      const html = await fetchPackDetailHtmlForItemId(itemId, 12000)
      const games = parsePackModal(html)
      const parentId = parentMap?.get(sid)
      if (parentId) {
        collectedGames.push(...games.map(g => ({ ...g, parentPackWorkId: parentId })))
      }
      else {
        collectedGames.push(...games)
      }
    }
    catch {}
    await new Promise(r => setTimeout(r, 500))
  }
  return collectedGames
}
