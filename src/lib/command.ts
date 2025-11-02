import type { AllGameCacheOne } from '@/lib/types'
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
import { invoke as invokeCore } from '@tauri-apps/api/core'

async function invoke<T>(command: Parameters<typeof invokeCore>[0], args?: Parameters<typeof invokeCore>[1]) {
  // eslint-disable-next-line no-console
  console.log('invoke', command, args)
  const response = await invokeCore<T>(command, args)
  // eslint-disable-next-line no-console
  console.log('response', command, response)
  return response
}

export async function commandCreateElementsInPc(exploreDirPaths: string[], useCache: boolean) {
  return await invoke<string[]>('create_elements_in_pc', {
    exploreDirPaths,
    useCache,
  })
}

export async function commandScanStart(roots: string[], useCache: boolean) {
  return await invoke<string[]>('scan_start', {
    roots,
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

export type WorkPathInput
  = | { type: 'exe', exePath: string }
    | { type: 'lnk', lnkPath: string }

export async function commandRegisterWorkFromPath(arg: {
  path: WorkPathInput
  gameCache: AllGameCacheOne
}) {
  return await invoke<void>('register_work_from_path', arg)
}

export async function commandGetDefaultImportDirs() {
  return await invoke<string[]>('get_default_import_dirs', {})
}

export async function commandLaunchWork(isRunAsAdmin: boolean, workLnkId: number) {
  return await invoke<number | null>('launch_work', {
    isRunAsAdmin,
    workLnkId,
  })
}

export async function commandListWorkLnks(workId: string) {
  return await invoke<[number, string][]>('list_work_lnks', { workId })
}

export async function commandGetPlayTomeMinutes(collectionElementId: number) {
  return await invoke<number>('get_play_time_minutes', { collectionElementId })
}

// removed: commandGetCollectionElement

// removed: commandDeleteCollectionElement
export async function commandDeleteWork(workId: string) {
  return await invoke<void>('delete_work', { workId })
}

// 詳細未登録の EGS ID 群を取得
export async function commandGetNotRegisteredErogamescapeInformationIds() {
  return await invoke<number[]>('get_not_registered_erogamescape_information_ids', {})
}

export interface ErogamescapeInformationInput {
  erogamescapeId: number
  gamenameRuby: string
  brandname: string
  brandnameRuby: string
  sellday: string
  isNukige: boolean
}
export async function commandUpsertErogamescapeInformation(details: ErogamescapeInformationInput[]) {
  return await invoke<void>('upsert_erogamescape_information', { details })
}

// removed: commandGetAllElements

export async function commandUpdateWorkLike(workId: string, isLike: boolean) {
  return await invoke<void>('update_work_like', { workId, isLike })
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

export async function commandOpenUrl(url: string) {
  return await invoke<void>('open_url', { url })
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

// Parent DMM Pack keys for a child work
export interface DmmPackKeysVm { storeId: string, category: string, subcategory: string }
export async function commandGetParentDmmPackKeys(workId: string) {
  return await invoke<DmmPackKeysVm | null>('get_parent_dmm_pack_keys', { workId })
}

// Native Messaging Host Logs
export interface HostLogItem {
  id: number
  level: number
  typ: number
  message: string
  created_at: string
}
export interface HostLogsResponse {
  items: HostLogItem[]
  total: number
}
export async function commandGetNativeHostLogs(req: { limit?: number, offset?: number, level?: number, typ?: number }) {
  return await invoke<HostLogsResponse>('get_native_host_logs', { request: req })
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

// Work Omit Commands
export async function commandWorkOmitAdd(workId: string) {
  return await invoke<void>('work_omit_add', { workId })
}
export async function commandWorkOmitRemove(workId: string) {
  return await invoke<void>('work_omit_remove', { workId })
}
export async function commandWorkOmitAll() {
  return await invoke<string[]>('work_omit_all')
}

// WorkDetails
export interface WorkDetailsVm { id: string, title: string, dmm?: { id: number, storeId: string, category: string, subcategory: string }, dlsite?: { id: number, storeId: string, category: string }, erogamescapeId?: number | null, erogamescapeInformation?: { gamenameRuby: string, brandname: string, brandnameRuby: string, sellday: string, isNukige: boolean }, isOmitted: boolean, isDmmPack: boolean, thumbnail?: { path: string, width?: number, height?: number } | null, latestDownloadPath?: { id: number, workId: string, downloadPath: string } | null, likeAt?: string | null, installAt?: string | null, lastPlayAt?: string | null, registeredAt?: string | null }
export async function commandGetWorkDetailsAll() {
  return await invoke<WorkDetailsVm[]>('get_work_details_all')
}

export async function commandGetWorkDetailsByWorkId(workId: string) {
  return await invoke<WorkDetailsVm | null>('get_work_details_by_work_id', { workId })
}

// Work Paths
export interface WorkLnkVm { id: number, lnkPath: string }
export interface WorkPathsVm { lnks: WorkLnkVm[] }
export async function commandGetWorkPaths(workId: string) {
  return await invoke<WorkPathsVm>('get_work_paths', { workId })
}

// DMM Work Packs wrappers
export async function commandWorkPackAll() {
  return await invoke<string[]>('work_pack_all')
}
export async function commandWorkPackAdd(workId: string) {
  return await invoke<void>('work_pack_add', { workId })
}
export async function commandWorkPackRemove(workId: string) {
  return await invoke<void>('work_pack_remove', { workId })
}

// Image Save Queue
export interface ImageSaveQueueRowVm {
  id: number
  src: string
  srcType: number
  dstPath: string
  preprocess: number
  lastError?: string | null
}
export async function commandGetImageSaveQueue(req?: { limit?: number, status?: 'unfinished' | 'finished' }) {
  return await invoke<ImageSaveQueueRowVm[]>('get_image_save_queue', req ? { request: req } : {})
}

// Backfill for missing thumbnail sizes
export async function commandBackfillThumbnailSizes() {
  return await invoke<number>('backfill_thumbnail_sizes')
}

// Process pending exe links (work_link_pending_exe)
export async function commandProcessPendingExeLinks() {
  return await invoke<void>('process_pending_exe_links')
}
