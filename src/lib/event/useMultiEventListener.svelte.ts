import type { UnlistenFn } from '@tauri-apps/api/event'
import type { EventName, EventPayloadMap, TypedEventHandler } from './types'
import { listen } from '@tauri-apps/api/event'

export interface TypedEventListener<T extends EventName = EventName> {
  eventName: T
  handler: TypedEventHandler<T>
}

export function useMultiEventListener() {
  const listeners = new Map<string, UnlistenFn>()

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

  const startMultipleListen = async (eventListeners: TypedEventListener[]) => {
    for (const { eventName, handler } of eventListeners) {
      await startListen(eventName, handler as TypedEventHandler<typeof eventName>)
    }
  }

  return {
    startListen,
    stopListen,
    stopAllListeners,
    startMultipleListen,
  }
}
