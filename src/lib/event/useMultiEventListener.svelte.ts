import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@tauri-apps/api/event'

export interface EventListener<T = any> {
  eventName: string
  handler: (payload: T) => void
}

export function useMultiEventListener() {
  const listeners = new Map<string, UnlistenFn>()

  const startListen = async <T>(eventName: string, handler: (payload: T) => void) => {
    // 既存のリスナーがあれば停止
    if (listeners.has(eventName)) {
      const existingUnlisten = listeners.get(eventName)!
      existingUnlisten()
    }

    const unlistenFn = await listen<T>(eventName, (event) => {
      handler(event.payload)
    })

    listeners.set(eventName, unlistenFn)
  }

  const stopListen = (eventName: string) => {
    const unlistenFn = listeners.get(eventName)
    if (unlistenFn) {
      unlistenFn()
      listeners.delete(eventName)
    }
  }

  const stopAllListeners = () => {
    for (const [_eventName, unlistenFn] of listeners) {
      unlistenFn()
    }
    listeners.clear()
  }

  const startMultipleListen = async (eventListeners: EventListener[]) => {
    for (const { eventName, handler } of eventListeners) {
      await startListen(eventName, handler)
    }
  }

  return {
    startListen,
    stopListen,
    stopAllListeners,
    startMultipleListen,
  }
}
