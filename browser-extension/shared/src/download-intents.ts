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

export type DownloadIntentEntry
  = | {
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

export type DownloadIntentMap = Record<string, DownloadIntentEntry>

export async function readAllDownloadIntents(): Promise<DownloadIntentMap> {
  try {
    return await new Promise<DownloadIntentMap>((resolve) => {
      chrome.storage.local.get([STORAGE_KEY], res => resolve((res?.[STORAGE_KEY] as DownloadIntentMap) ?? {}))
    })
  }
  catch {
    return {}
  }
}

export async function writeAllDownloadIntents(map: DownloadIntentMap): Promise<void> {
  await chrome.storage.local.set({ [STORAGE_KEY]: map })
}

export async function getDownloadIntent(storeId: string): Promise<DownloadIntentEntry | null> {
  const map = await readAllDownloadIntents()
  return map[storeId] ?? null
}

export async function setDownloadIntent(storeId: string, entry: DownloadIntentEntry): Promise<void> {
  const map = await readAllDownloadIntents()
  map[storeId] = entry
  await writeAllDownloadIntents(map)
}

export async function removeDownloadIntent(storeId: string): Promise<void> {
  const map = await readAllDownloadIntents()
  if (storeId in map)
    delete map[storeId]
  await writeAllDownloadIntents(map)
}

export async function clearDownloadIntents(): Promise<void> {
  await writeAllDownloadIntents({})
}

export async function incrementCompletedAndPushItem(storeId: string, item: DownloadCompletedTs): Promise<DownloadIntentEntry | null> {
  const current = await getDownloadIntent(storeId)
  if (!current)
    return null
  const next: DownloadIntentEntry = {
    ...current,
    completed: (current.completed ?? 0) + 1,
    items: [...(current.items ?? []), item],
  }
  await setDownloadIntent(storeId, next)
  return next
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
        game_category: String((entry as any).game?.category ?? ''),
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
