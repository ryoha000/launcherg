import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@tauri-apps/api/event'

export interface EventListener<T = any> {
  eventName: string
  handler: (payload: T) => void
}

export function useEventListener<T = any>() {
  let unlistenFn: UnlistenFn | null = null

  const startListen = async (eventName: string, handler: (payload: T) => void) => {
    if (unlistenFn) {
      unlistenFn()
    }

    unlistenFn = await listen<T>(eventName, (event) => {
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
