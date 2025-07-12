import type {
  AllGameCacheOne,
  CollectionElement,
  CollectionElementDetail,
} from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'

export async function commandCreateElementsInPc(exploreDirPaths: string[], useCache: boolean) {
  return await invoke<string[]>('create_elements_in_pc', {
    exploreDirPaths,
    useCache,
  })
}

export async function commandGetNearestKeyAndDistance(key: string, calculateDistanceKv: [string, string][]) {
  return await invoke<[string, number]>('get_nearest_key_and_distance', {
    key,
    calculateDistanceKv,
  })
}

export async function commandUploadImage(id: number, base64Image: string) {
  return await invoke<string>('upload_image', {
    id,
    base64Image,
  })
}

export async function commandUpsertCollectionElement(arg: {
  exePath: string | null
  lnkPath: string | null
  gameCache: AllGameCacheOne
}) {
  return await invoke<void>('upsert_collection_element', arg)
}

export async function commandUpdateCollectionElementIcon(id: number, path: string) {
  return await invoke<void>('update_collection_element_icon', {
    id,
    path,
  })
}

export async function commandGetDefaultImportDirs() {
  return await invoke<string[]>('get_default_import_dirs', {})
}

export async function commandPlayGame(collectionElementId: number, isRunAsAdmin: boolean) {
  return await invoke<number | null>('play_game', {
    collectionElementId,
    isRunAsAdmin,
  })
}

export async function commandGetPlayTomeMinutes(collectionElementId: number) {
  return await invoke<number>('get_play_time_minutes', { collectionElementId })
}

export async function commandGetCollectionElement(collectionElementId: number) {
  return await invoke<CollectionElement>('get_collection_element', {
    collectionElementId,
  })
}

export async function commandDeleteCollectionElement(collectionElementId: number) {
  return await invoke<void>('delete_collection_element', {
    collectionElementId,
  })
}

export async function commandGetNotRegisterdDetailElementIds() {
  return await invoke<number[]>('get_not_registered_detail_element_ids', {})
}

export async function commandCreateElementDetails(details: CollectionElementDetail[]) {
  return await invoke<void>('create_element_details', {
    details,
  })
}

export async function commandGetAllElements() {
  return await invoke<CollectionElement[]>('get_all_elements', {})
}

export async function commandUpdateElementLike(id: number, isLike: boolean) {
  return await invoke<void>('update_element_like', { id, isLike })
}

export async function commandOpenFolder(path: string) {
  return await invoke<void>('open_folder', { path })
}

export async function commandGetAllGameCacheLastUpdated() {
  const [id, dateString] = await invoke<[number, string]>(
    'get_all_game_cache_last_updated',
  )
  return { id, date: new Date(dateString) }
}

export async function commandUpdateAllGameCache(gameCaches: AllGameCacheOne[]) {
  await invoke('update_all_game_cache', {
    gameCaches,
  })
}

export async function commandGetGameCandidates(filepath: string) {
  return await invoke<[number, string][]>('get_game_candidates', {
    filepath,
  })
}

export async function commandGetExePathByLnk(filepath: string) {
  return await invoke<string>('get_exe_path_by_lnk', {
    filepath,
  })
}

export async function commandGetGameCacheById(id: number) {
  return await invoke<AllGameCacheOne | null>('get_game_cache_by_id', {
    id,
  })
}

export async function commandSaveScreenshotByPid(workId: number, processId: number) {
  return await invoke<string>('save_screenshot_by_pid', {
    workId,
    processId,
  })
}

export async function commandUpdateCollectionElementThumbnails(ids: number[]) {
  return await invoke<void>('update_collection_element_thumbnails', {
    ids,
  })
}
