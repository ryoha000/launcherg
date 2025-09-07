// Tauri イベントの型定義

export interface ProgressPayload {
  message: string
}

export interface ProgressLivePayload {
  max: number | null
}

export interface ScanEnrichResultPayload {
  status: 'candidate' | 'resolved'
  path: string
  title?: string | null
  egsId?: number | null
}

export interface ScanDedupPayload {
  removedCount: number
}

// イベント名とペイロードの型マッピング
export interface EventPayloadMap {
  progress: ProgressPayload
  progresslive: ProgressLivePayload
  scanEnrichResult: ScanEnrichResultPayload
  scanDedup: ScanDedupPayload
}

// イベント名の型
export type EventName = keyof EventPayloadMap

// 型安全なイベントハンドラー
export type TypedEventHandler<T extends EventName> = (payload: EventPayloadMap[T]) => void
