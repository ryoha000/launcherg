// Tauri イベントの型定義

export interface ProgressPayload {
  message: string
}

export interface ProgressLivePayload {
  max: number | null
}

// イベント名とペイロードの型マッピング
export interface EventPayloadMap {
  progress: ProgressPayload
  progresslive: ProgressLivePayload
}

// イベント名の型
export type EventName = keyof EventPayloadMap

// 型安全なイベントハンドラー
export type TypedEventHandler<T extends EventName> = (payload: EventPayloadMap[T]) => void
