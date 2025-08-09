import type {
  AllGameCacheOne,
  CollectionElement,
  CollectionElementDetail,
} from '@/lib/types'
import type {
  AddWatchTargetRequest,
  ClearEventsRequest,
  GetEventsRequest,
  HealthCheckResult,
  ProcTailEvent,
  ProcTailManagerStatus,
  ProcTailVersion,
  RemoveWatchTargetRequest,
  ServiceStatus,
  WatchTarget,
} from '@/lib/types/proctail'
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

export async function commandGetGameCandidatesByName(gameName: string) {
  return await invoke<[number, string][]>('get_game_candidates_by_name', { gameName })
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

// ProcTail Commands
export async function commandProcTailAddWatchTarget(request: AddWatchTargetRequest) {
  return await invoke<WatchTarget>('proctail_add_watch_target', { request })
}

export async function commandProcTailRemoveWatchTarget(request: RemoveWatchTargetRequest) {
  return await invoke<number>('proctail_remove_watch_target', { request })
}

export async function commandProcTailGetWatchTargets() {
  return await invoke<WatchTarget[]>('proctail_get_watch_targets')
}

export async function commandProcTailGetRecordedEvents(request: GetEventsRequest) {
  return await invoke<ProcTailEvent[]>('proctail_get_recorded_events', { request })
}

export async function commandProcTailClearEvents(request: ClearEventsRequest) {
  return await invoke<number>('proctail_clear_events', { request })
}

export async function commandProcTailGetStatus() {
  return await invoke<ServiceStatus>('proctail_get_status')
}

export async function commandProcTailHealthCheck() {
  return await invoke<HealthCheckResult>('proctail_health_check')
}

export async function commandProcTailIsServiceAvailable() {
  return await invoke<boolean>('proctail_is_service_available')
}

// ProcTail Manager Commands
export async function commandProcTailManagerGetStatus() {
  return await invoke<ProcTailManagerStatus>('proctail_manager_get_status')
}

export async function commandProcTailManagerGetLatestVersion() {
  return await invoke<ProcTailVersion>('proctail_manager_get_latest_version')
}

export async function commandProcTailManagerIsUpdateAvailable() {
  return await invoke<boolean>('proctail_manager_is_update_available')
}

export async function commandProcTailManagerDownloadAndInstall() {
  return await invoke<void>('proctail_manager_download_and_install')
}

export async function commandProcTailManagerStart() {
  return await invoke<void>('proctail_manager_start')
}

export async function commandProcTailManagerStop() {
  return await invoke<void>('proctail_manager_stop')
}

export async function commandProcTailManagerIsRunning() {
  return await invoke<boolean>('proctail_manager_is_running')
}

// DL版ゲーム管理機能のコマンド
export async function commandRegisterDLStoreGame(
  storeType: 'DMM' | 'DLSite',
  storeId: string,
  erogamescapeId: number | null,
  purchaseUrl: string,
) {
  return await invoke<number>('register_dl_store_game', {
    storeType,
    storeId,
    erogamescapeId,
    purchaseUrl,
  })
}

export async function commandOpenStorePage(purchaseUrl: string) {
  return await invoke<void>('open_store_page', { purchaseUrl })
}

export async function commandLinkInstalledGame(
  collectionElementId: number,
  exePath: string,
) {
  return await invoke<void>('link_installed_game', {
    collectionElementId,
    exePath,
  })
}

export async function commandGetUninstalledOwnedGames() {
  return await invoke<CollectionElement[]>('get_uninstalled_owned_games')
}

export async function commandUpdateDLStoreOwnership(
  dlStoreId: number,
  isOwned: boolean,
) {
  return await invoke<void>('update_dl_store_ownership', {
    dlStoreId,
    isOwned,
  })
}

// 拡張機能連携用の新しいコマンド

export async function commandGetSyncStatus() {
  return await invoke<any>('get_sync_status')
}

export async function commandSetExtensionConfig(config: any) {
  return await invoke<void>('set_extension_config', { config })
}

// 拡張機能インストーラー関連の型定義
interface ExtensionManifestInfo {
  name: string
  version: string
  extension_id: string
  description: string
}

interface ExtensionPackageInfo {
  version: string
  package_path: string
  manifest_info: ExtensionManifestInfo
}

export async function commandGenerateExtensionPackage() {
  return await invoke<ExtensionPackageInfo>('generate_extension_package')
}

export async function commandSetupNativeMessagingHost(options?: { extensionId?: string }) {
  return await invoke<string>('setup_native_messaging_host', options || {})
}

export async function commandGetExtensionPackageInfo() {
  return await invoke<ExtensionPackageInfo | null>('get_extension_package_info')
}

export async function commandCopyExtensionForDevelopment() {
  return await invoke<string>('copy_extension_for_development')
}

export async function commandGetDevExtensionInfo() {
  return await invoke<string | null>('get_dev_extension_info')
}

export interface RegistryKeyInfo {
  browser: string
  key_path: string
  value: string | null
  exists: boolean
}

export async function commandCheckRegistryKeys() {
  return await invoke<RegistryKeyInfo[]>('check_registry_keys')
}

export async function commandRemoveRegistryKeys() {
  return await invoke<string[]>('remove_registry_keys')
}
