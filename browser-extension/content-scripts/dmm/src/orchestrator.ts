import type { DmmExtractedGame } from './types'
import { create, fromJson, toJson } from '@bufbuild/protobuf'

import { sendExtensionRequest } from '@launcherg/shared'
import { DmmGameSchema, DmmSyncGamesRequestSchema, ExtensionRequestSchema, GetDmmPackIdsRequestSchema as ExtGetPacksReq, GetDmmPackIdsResponseSchema as ExtGetPacksRes } from '@launcherg/shared/proto/extension_internal'
import { fetchPackDetailHtmlForItemId, findDetailItemIdForStoreId } from './pack-helpers'
import { parsePackModal } from './pack-parser'

export async function fetchPackIds(): Promise<Set<string>> {
  const packReq = create(ExtensionRequestSchema, {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'getDmmPackIds', value: create(ExtGetPacksReq, {}) },
  } as any)
  const packResJson = await sendExtensionRequest(packReq as any, (req: any) => toJson(ExtensionRequestSchema, req as any))
  const raw = (packResJson as any)?.response?.value ?? (packResJson as any)?.getDmmPackIdsResult ?? {}
  const packRes = fromJson(ExtGetPacksRes, raw)
  return new Set<string>(packRes.storeIds)
}

export async function syncDmmGames(games: DmmExtractedGame[]): Promise<void> {
  if (games.length === 0)
    return
  const dmmGames = games.map(g => create(DmmGameSchema, {
    id: g.storeId,
    category: g.category,
    subcategory: g.subcategory,
    title: g.title,
    imageUrl: g.imageUrl,
  }))
  const request = create(ExtensionRequestSchema, {
    requestId: Date.now().toString(36) + Math.random().toString(36).slice(2),
    request: { case: 'syncDmmGames', value: create(DmmSyncGamesRequestSchema, { games: dmmGames }) },
  })
  await sendExtensionRequest(request, req => toJson(ExtensionRequestSchema, req))
}

export async function processPacks(packSet: Set<string>): Promise<DmmExtractedGame[]> {
  const packIds = Array.from(packSet)
  const collectedGames: DmmExtractedGame[] = []
  for (const sid of packIds) {
    const itemId = findDetailItemIdForStoreId(sid)
    if (!itemId)
      continue
    try {
      const html = await fetchPackDetailHtmlForItemId(itemId, 12000)
      const games = parsePackModal(html)
      collectedGames.push(...games)
    }
    catch {}
    await new Promise(r => setTimeout(r, 500))
  }
  return collectedGames
}
