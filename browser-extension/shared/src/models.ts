// Extension Internal communication models (TypeScript-only)

// EGS 解決済み情報
export interface EgsInfo {
  erogamescapeId: number
  gamename: string
  gamenameRuby: string
  brandname: string
  brandnameRuby: string
  sellday: string
  isNukige: boolean
}

// DMM / DLsite のゲームデータ
export interface DmmGame {
  id: string
  category: string // "mono" | "digital"
  subcategory: string // "doujin" | "pcgame"
  egsInfo?: EgsInfo
  title: string
  imageUrl: string
  parentPackWorkId?: number
}

export interface DlsiteGame {
  id: string
  category: string // domain, e.g. "ai" | "maniax" | ...
  egsInfo?: EgsInfo
  title: string
  imageUrl: string
}

// リクエスト系
export interface DmmSyncGamesRequest { games: DmmGame[] }
export interface DlsiteSyncGamesRequest { games: DlsiteGame[] }
export interface GetStatusRequest {}
export interface DebugNativeMessageRequest { payloadJson: string }
export interface GetDmmOmitWorksRequest {}

// レスポンス系
export interface SyncResult {
  successCount: number
  errorCount: number
  errors: string[]
  syncedGames: string[]
}

export interface SyncGamesResponse {
  result?: SyncResult
  message: string
}

export interface StatusData {
  lastSync: string
  totalSynced: number
  connectedExtensions: string[]
  isRunning: boolean
  connectionStatus: string
  errorMessage: string
}

export interface GetStatusResponse { status?: StatusData }

export interface DebugNativeMessageResponse {
  nativeResponseJson: string
  timestamp: string
}

export interface GetDmmOmitWorksResponse { items: { workId: number, dmm: { storeId: string, category: string, subcategory: string } }[] }

// Extension Request / Response (oneof 相当を discriminated union 的に表現)
export interface ExtensionRequest {
  requestId: string
  request:
    | { case: 'syncDmmGames', value: DmmSyncGamesRequest }
    | { case: 'syncDlsiteGames', value: DlsiteSyncGamesRequest }
    | { case: 'getStatus', value: GetStatusRequest }
    | { case: 'debugNativeMessage', value: DebugNativeMessageRequest }
    | { case: 'getDmmOmitWorks', value: GetDmmOmitWorksRequest }
    | { case: undefined }
}

export interface ExtensionResponse {
  requestId: string
  success: boolean
  error: string
  response:
    | { case: 'syncGamesResult', value: SyncGamesResponse }
    | { case: 'statusResult', value: GetStatusResponse }
    | { case: 'debugResult', value: DebugNativeMessageResponse }
    | { case: 'getDmmOmitWorksResult', value: GetDmmOmitWorksResponse }
    | { case: undefined }
}
