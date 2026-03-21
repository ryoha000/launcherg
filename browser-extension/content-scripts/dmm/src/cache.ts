import type { DmmExtractedGame } from './types'

type CachedChild = Omit<DmmExtractedGame, 'parentPack'>

function buildChildrenKey(storeId: string): string {
  return `dmm:pack:children:${storeId}`
}

export async function getCachedPackChildren(storeId: string): Promise<CachedChild[] | null> {
  try {
    const key = buildChildrenKey(storeId)
    const obj = await chrome.storage.local.get(key)
    const entry = obj[key] as { children?: CachedChild[] } | undefined
    if (entry && Array.isArray(entry.children))
      return entry.children
    return null
  }
  catch {
    return null
  }
}

export async function getCachedPackChildrenMulti(storeIds: string[]): Promise<Map<string, CachedChild[]>> {
  const result = new Map<string, CachedChild[]>()
  if (storeIds.length === 0)
    return result
  try {
    const keys = storeIds.map(buildChildrenKey)
    const obj = await chrome.storage.local.get(keys)
    for (const sid of storeIds) {
      const key = buildChildrenKey(sid)
      const entry = obj[key] as { children?: CachedChild[] } | undefined
      if (entry && Array.isArray(entry.children))
        result.set(sid, entry.children)
    }
  }
  catch {}
  return result
}

export async function setCachedPackChildren(storeId: string, children: CachedChild[]): Promise<void> {
  try {
    const key = buildChildrenKey(storeId)
    await chrome.storage.local.set({ [key]: { children } })
  }
  catch {}
}
