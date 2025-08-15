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
    },
    aggregation: {
      record: async () => {},
    },
    idGenerator: {
      generate: () => 'test-request-id',
    },
  }
  return { ...base, ...overrides }
}
