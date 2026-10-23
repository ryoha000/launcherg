import type { DownloadCompletedTs, DownloadIntentTs } from './typeshare/native-messaging'

const STORAGE_KEY = 'download_intents'

export type DownloadIntentStore = 'DMM' | 'DLsite'

export interface DmmIntentGame {
  storeId: string
  category: string
  subcategory: string
}

export interface DmmIntentParentPack {
  storeId: string
  category: string
  subcategory: string
}

export interface DlsiteIntentGame {
  storeId: string
  // カテゴリは現状未保存のことがあるため任意
  category?: string
}

export type DownloadIntentEntry = {
  attempts?: number
  lastError?: string
  nextRetryAt?: number
  downloadIds?: number[]
  pendingUrls?: string[]
} & (
  | {
    store: 'DMM'
    game: DmmIntentGame
    parentPack?: DmmIntentParentPack
    expected: number
    completed: number
    startedAt: number
    items?: DownloadCompletedTs[]
  }
  | {
    store: 'DLsite'
    game: DlsiteIntentGame
    expected: number
    completed: number
    startedAt: number
    items?: DownloadCompletedTs[]
  }

)

export type DownloadIntentMap = Record<string, DownloadIntentEntry>

// Mutations are serialized in the background worker, including registration from tabs.
let mutationTail: Promise<unknown> = Promise.resolve()
function mutate<T>(fn: (map: DownloadIntentMap) => Promise<T>): Promise<T> {
  const current = mutationTail.catch(() => undefined).then(async () => fn(await readAllDownloadIntents()))
  mutationTail = current
  return current
}

export async function readAllDownloadIntents(): Promise<DownloadIntentMap> {
  const result = await chrome.storage.local.get([STORAGE_KEY])
  return (result[STORAGE_KEY] as DownloadIntentMap | undefined) ?? {}
}

export async function getDownloadIntent(storeId: string): Promise<DownloadIntentEntry | null> {
  return (await readAllDownloadIntents())[storeId] ?? null
}

export async function setDownloadIntent(storeId: string, entry: DownloadIntentEntry): Promise<void> {
  const response = await chrome.runtime.sendMessage({ type: 'register_download_intent', storeId, entry })
  if (!response?.success)
    throw new Error(response?.error || 'ダウンロード情報を保存できませんでした')
}

export async function registerDownloadIntent(storeId: string, entry: DownloadIntentEntry): Promise<void> {
  if (!storeId || entry.game?.storeId !== storeId || !Number.isInteger(entry.expected) || entry.expected < 1
    || !['DMM', 'DLsite'].includes(entry.store) || !Number.isFinite(entry.startedAt)) {
    throw new Error('ダウンロード情報が正しくありません')
  }
  await mutate(async (map) => {
    if (map[storeId])
      throw new Error('この作品には未完了のダウンロードがあります。拡張機能から再試行してください')
    map[storeId] = { ...entry, completed: 0, items: [], attempts: 0 }
    await chrome.storage.local.set({ [STORAGE_KEY]: map })
  })
}

export async function updateDownloadIntent(storeId: string, patch: Partial<DownloadIntentEntry>): Promise<void> {
  await mutate(async (map) => {
    if (map[storeId]) {
      map[storeId] = { ...map[storeId], ...patch } as DownloadIntentEntry
      await chrome.storage.local.set({ [STORAGE_KEY]: map })
    }
  })
}

export async function removeDownloadIntent(storeId: string): Promise<void> {
  await mutate(async (map) => {
    delete map[storeId]
    await chrome.storage.local.set({ [STORAGE_KEY]: map })
  })
}

export async function discardFailedDownloadIntent(storeId: string): Promise<void> {
  await mutate(async (map) => {
    if (!map[storeId]?.lastError)
      throw new Error('失敗した処理だけ解除できます')
    delete map[storeId]
    await chrome.storage.local.set({ [STORAGE_KEY]: map })
  })
}

export async function incrementCompletedAndPushItem(storeId: string, item: DownloadCompletedTs): Promise<DownloadIntentEntry | null> {
  return mutate(async (map) => {
    const current = map[storeId]
    if (!current)
      return null
    const items = [...(current.items ?? []).filter(existing => existing.id !== item.id), item]
    const next: DownloadIntentEntry = { ...current, completed: items.length, items }
    map[storeId] = next
    await chrome.storage.local.set({ [STORAGE_KEY]: map })
    return next
  })
}

export function toDownloadIntentTs(entry: DownloadIntentEntry): DownloadIntentTs | undefined {
  if (entry.store === 'DMM') {
    return {
      case: 'Dmm',
      value: {
        game_store_id: String(entry.game.storeId ?? ''),
        game_category: String(entry.game.category ?? ''),
        game_subcategory: String(entry.game.subcategory ?? ''),
        parent_pack_store_id: entry.parentPack?.storeId ?? undefined,
        parent_pack_category: entry.parentPack?.category ?? undefined,
        parent_pack_subcategory: entry.parentPack?.subcategory ?? undefined,
      },
    }
  }
  if (entry.store === 'DLsite') {
    return {
      case: 'Dlsite',
      value: {
        game_store_id: String(entry.game.storeId ?? ''),
        game_category: String(entry.game.category ?? ''),
      },
    }
  }
  return undefined
}

export function stripDownloadItemFields(item: chrome.downloads.DownloadItem): DownloadCompletedTs {
  return {
    id: item.id,
    filename: item.filename ?? '',
    mime: item.mime,
    url: item.url,
    start_time: item.startTime,
    end_time: item.endTime,
  }
}
