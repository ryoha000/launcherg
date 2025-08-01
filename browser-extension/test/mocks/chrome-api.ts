import { vi } from 'vitest'

export function setupChromeMocks() {
  const mockStorage = {
    local: {
      get: vi.fn().mockResolvedValue({}),
      set: vi.fn().mockResolvedValue(undefined),
      remove: vi.fn().mockResolvedValue(undefined),
      clear: vi.fn().mockResolvedValue(undefined),
    },
    sync: {
      get: vi.fn().mockResolvedValue({}),
      set: vi.fn().mockResolvedValue(undefined),
      remove: vi.fn().mockResolvedValue(undefined),
      clear: vi.fn().mockResolvedValue(undefined),
    },
  }

  const mockRuntime = {
    id: 'test-extension-id',
    lastError: null,
    sendMessage: vi.fn((message, callback) => {
      if (callback) {
        // デフォルトのレスポンス
        const response = {
          success: true,
          response: {
            case: 'statusResult',
            value: {
              status: {
                lastSync: new Date().toISOString(),
                totalSynced: 10,
              },
            },
          },
        }
        callback(response)
      }
      return true
    }),
    onMessage: {
      addListener: vi.fn(),
      removeListener: vi.fn(),
      hasListener: vi.fn().mockReturnValue(false),
    },
  }

  const mockTabs = {
    query: vi.fn().mockResolvedValue([
      {
        id: 1,
        url: 'https://games.dmm.co.jp/test',
        title: 'Test Page',
      },
    ]),
    sendMessage: vi.fn((tabId, message, callback) => {
      if (callback) {
        callback({ success: true, message: 'Test response' })
      }
      return true
    }),
    create: vi.fn().mockResolvedValue({
      id: 2,
      url: 'https://example.com',
    }),
  }

  // グローバルな chrome オブジェクトを設定
  ;(globalThis as any).chrome = {
    storage: mockStorage,
    runtime: mockRuntime,
    tabs: mockTabs,
  }

  // navigator.clipboard のモック
  Object.defineProperty(navigator, 'clipboard', {
    value: {
      writeText: vi.fn().mockResolvedValue(undefined),
      readText: vi.fn().mockResolvedValue(''),
    },
    writable: true,
  })
}
