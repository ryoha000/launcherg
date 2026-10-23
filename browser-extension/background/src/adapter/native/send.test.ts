import { afterEach, describe, expect, it, vi } from 'vitest'
import { createNativeMessenger } from './send'

const host = 'com.example.native'
afterEach(() => vi.useRealTimers())

function mockChromeSendNativeMessage(response: unknown, lastError?: string) {
  Object.defineProperty(globalThis as any, 'chrome', {
    configurable: true,
    value: {
      runtime: {
        lastError: lastError ? { message: lastError } : undefined,
        sendNativeMessage: vi.fn((_host: string, _payload: any, cb: (res: unknown) => void) => cb(response)),
      },
    },
  })
}

describe('ネイティブメッセンジャー送受信(JSON)', () => {
  it('展開のタイムアウト時は接続を閉じてエラーを返す', async () => {
    vi.useFakeTimers()
    const disconnect = vi.fn()
    vi.stubGlobal('chrome', { runtime: { connectNative: () => ({
      onMessage: { addListener: vi.fn() },
      onDisconnect: { addListener: vi.fn() },
      postMessage: vi.fn(),
      disconnect,
    }) } })
    const pending = createNativeMessenger(host).sendJson({ request_id: 'timeout', message: {
      case: 'DownloadsCompleted',
      value: { extension_id: 'test', items: [], intent: { case: 'Dlsite', value: { game_store_id: 'RJ1', game_category: 'maniax' } } },
    } })
    const assertion = expect(pending).rejects.toThrow('timeout')
    await vi.advanceTimersByTimeAsync(20 * 60_000)
    await assertion
    expect(disconnect).toHaveBeenCalledOnce()
  })
  it('chrome.runtime.lastError が設定されている場合は reject する', async () => {
    mockChromeSendNativeMessage(undefined, 'boom')
    const messenger = createNativeMessenger(host)
    await expect(messenger.sendJson({ request_id: 'rid-3', message: { case: 'HealthCheck', value: {} } })).rejects.toThrow('boom')
  })

  it('null レスポンスは resolve(null) を返す', async () => {
    mockChromeSendNativeMessage(null)
    const messenger = createNativeMessenger(host)
    const res = await messenger.sendJson({ request_id: 'rid-4', message: { case: 'HealthCheck', value: {} } })
    expect(res).toBeNull()
  })
})
