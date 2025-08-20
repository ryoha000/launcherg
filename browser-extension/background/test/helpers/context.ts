import type { HandlerContext } from '../../src/shared/types'

export function buildTestContext(overrides: Partial<HandlerContext> = {}): HandlerContext {
  const base: HandlerContext = {
    extensionId: 'test-extension-id',
    nativeHostName: 'test-native-host',
    nativeMessenger: {
      send: async () => null,
    },
    egsResolver: {
      resolveForDmm: async () => null,
      resolveForDlsite: async () => null,
      resolveForDmmBulk: async (items: Array<{ storeId: string, category: string, subcategory: string }>) => items.map(() => null),
      resolveForDlsiteBulk: async (items: Array<{ storeId: string, category: string }>) => items.map(() => null),
    },
    idGenerator: {
      generate: () => 'test-request-id',
    },
    browser: {
      alarms: { create: async () => {} },
      notifications: { create: async () => {} },
      runtime: { getURL: (p: string) => `chrome-extension://test/${p}` },
      storage: {
        get: async () => ({}),
        set: async () => {},
      },
      tabs: {
        query: async () => [],
        sendMessage: async () => {},
      },
      scripting: {
        executeScript: async () => {},
      },
    },
    syncPool: {
      add: () => {},
      sync: async () => {},
    },
  }
  return { ...base, ...overrides }
}
