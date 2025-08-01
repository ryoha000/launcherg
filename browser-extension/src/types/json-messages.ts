// JSON メッセージ形式の型定義（従来形式との互換性維持）

// Content Script から Background Script へのメッセージ
export interface JsonMessage {
  type: string
  request_id?: string
  [key: string]: any
}

// ゲーム同期メッセージ
export interface JsonSyncGamesMessage extends JsonMessage {
  type: 'sync_games'
  store: string
  games: JsonGameData[]
  source?: string
}

export interface JsonGameData {
  store_id?: string
  title?: string
  purchase_url?: string
  purchase_date?: string
  thumbnail_url?: string
  additional_data?: Record<string, any>
}

// 設定取得メッセージ
export interface JsonGetConfigMessage extends JsonMessage {
  type: 'get_config'
  site: string
}

// 通知表示メッセージ
export interface JsonShowNotificationMessage extends JsonMessage {
  type: 'show_notification'
  title: string
  message: string
  iconType?: string
}

// ステータス取得メッセージ
export interface JsonGetStatusMessage extends JsonMessage {
  type: 'get_status'
}

// デバッグメッセージ
export interface JsonDebugNativeMessageMessage extends JsonMessage {
  type: 'debug_native_message'
  payload?: any
}

// レスポンス型
export interface JsonResponse {
  success: boolean
  error?: string
  [key: string]: any
}

export interface JsonSyncResponse extends JsonResponse {
  result?: {
    successCount?: number
    errorCount?: number
    errors?: string[]
    syncedGames?: string[]
  }
  message?: string
}

export interface JsonConfigResponse extends JsonResponse {
  config?: any
}

export interface JsonStatusResponse extends JsonResponse {
  status?: {
    lastSync?: string
    totalSynced?: number
    connectedExtensions?: string[]
    isRunning?: boolean
    connectionStatus?: string
    errorMessage?: string
  }
}

export interface JsonDebugResponse extends JsonResponse {
  native_response?: any
  timestamp?: string
}
