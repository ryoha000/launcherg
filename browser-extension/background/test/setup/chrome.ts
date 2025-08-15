/* eslint vars-on-top: off */
import { vi } from 'vitest'

// Minimal chrome API mocks used in background tests
const alarms = {
  create: vi.fn(),
  onAlarm: { addListener: vi.fn() },
}

const runtime = {
  id: 'test-extension-id',
  getURL: vi.fn((path: string) => `chrome-extension://test/${path}`),
  onInstalled: { addListener: vi.fn() },
  onMessage: { addListener: vi.fn() },
}

const tabs = {
  query: vi.fn(async () => [] as any[]),
  sendMessage: vi.fn(async () => {}),
  onUpdated: { addListener: vi.fn() },
}

const notifications = {
  create: vi.fn(async () => {}),
}

const scripting = {
  executeScript: vi.fn(async () => {}),
}

const storage = {
  local: {
    get: vi.fn((_: any, cb: (items: Record<string, any>) => void) => {
      cb({})
    }),
    set: vi.fn((_: any, cb?: () => void) => cb && cb()),
  },
}

;(globalThis as any).chrome = {
  alarms,
  runtime,
  tabs,
  notifications,
  scripting,
  storage,
}

// helper to reset between tests if needed
export function resetChromeMocks() {
  alarms.create.mockReset()
  ;(alarms.onAlarm.addListener as any).mockReset?.()
  runtime.getURL.mockReset()
  ;(runtime.onInstalled.addListener as any).mockReset?.()
  ;(runtime.onMessage.addListener as any).mockReset?.()
  tabs.query.mockReset()
  tabs.sendMessage.mockReset()
  ;(tabs.onUpdated.addListener as any).mockReset?.()
  notifications.create.mockReset()
  scripting.executeScript.mockReset()
  storage.local.get.mockReset()
  storage.local.set.mockReset()
}
