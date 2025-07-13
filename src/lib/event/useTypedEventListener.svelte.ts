import type { EventName, TypedEventHandler } from './types'
import { useEventListener } from './useEventListener.svelte'

/**
 * より強い型制約を持つEventListenerのラッパー
 * 使用例:
 * const progressListener = useTypedEventListener('progress')
 * await progressListener.startListen((payload) => {
 *   // payload は自動的に ProgressPayload 型になる
 *   console.log(payload.message)
 * })
 */
export function useTypedEventListener<T extends EventName>(eventName: T) {
  const eventListener = useEventListener()

  const startListen = async (handler: TypedEventHandler<T>) => {
    await eventListener.startListen(eventName, handler)
  }

  const stopListen = () => {
    eventListener.stopListen()
  }

  return {
    eventName,
    startListen,
    stopListen,
  }
}
