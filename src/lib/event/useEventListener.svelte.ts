import type { UnlistenFn } from '@tauri-apps/api/event'
import type { EventName, EventPayloadMap, TypedEventHandler } from './types'
import { listen } from '@tauri-apps/api/event'

export function useEventListener() {
  let unlistenFn: UnlistenFn | null = null

  const startListen = async <T extends EventName>(
    eventName: T,
    handler: TypedEventHandler<T>,
  ) => {
    if (unlistenFn) {
      unlistenFn()
    }

    unlistenFn = await listen<EventPayloadMap[T]>(eventName, (event) => {
      handler(event.payload)
    })
  }

  const stopListen = () => {
    if (unlistenFn) {
      unlistenFn()
      unlistenFn = null
    }
  }

  return {
    startListen,
    stopListen,
  }
}
