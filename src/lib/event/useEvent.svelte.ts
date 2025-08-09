import type { UnlistenFn } from '@tauri-apps/api/event'
import type { EventName, EventPayloadMap, TypedEventHandler } from './types'
import { listen } from '@tauri-apps/api/event'

/**
 * 型安全なTauriイベントリスナー
 * シンプルで使いやすい単一のAPI
 */
export function useEvent() {
  const listeners = new Map<string, UnlistenFn>()

  /**
   * イベントリスナーを開始
   */
  const startListen = async <T extends EventName>(
    eventName: T,
    handler: TypedEventHandler<T>,
  ) => {
    // 既存のリスナーがあれば停止
    if (listeners.has(eventName)) {
      const existingUnlisten = listeners.get(eventName)!
      existingUnlisten()
    }

    const unlistenFn = await listen<EventPayloadMap[T]>(eventName, (event) => {
      handler(event.payload)
    })

    listeners.set(eventName, unlistenFn)
  }

  /**
   * 特定のイベントリスナーを停止
   */
  const stopListen = (eventName: string) => {
    const unlistenFn = listeners.get(eventName)
    if (unlistenFn) {
      unlistenFn()
      listeners.delete(eventName)
    }
  }

  /**
   * すべてのイベントリスナーを停止
   */
  const stopAll = () => {
    for (const [, unlistenFn] of listeners) {
      unlistenFn()
    }
    listeners.clear()
  }

  return {
    startListen,
    stopListen,
    stopAll,
  }
}
