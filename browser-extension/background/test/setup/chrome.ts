/* eslint vars-on-top: off */
import { vi } from 'vitest'

// Minimal chrome API mocks used in background tests
const alarms = {
  create: vi.fn(),
  onAlarm: { addListener: vi.fn() },
}

export type CallbackSendNativeMessage = (application: string, message: object, responseCallback: (response: any) => void) => void
export const sendNativeMessageMock = vi.fn<CallbackSendNativeMessage>()

const getURLMock = vi.fn<(path: string) => string>((path: string) => `chrome-extension://test/${path}`)

const runtime = {
  id: 'test-extension-id',
  getURL: getURLMock,
  onInstalled: { addListener: vi.fn() },
  onMessage: { addListener: vi.fn() },
  sendNativeMessage: sendNativeMessageMock as unknown as typeof chrome.runtime.sendNativeMessage,
  lastError: undefined as chrome.runtime.LastError | undefined,
}

const tabs = {
  query: vi.fn<() => Promise<chrome.tabs.Tab[]>>(async () => []),
  sendMessage: vi.fn<() => Promise<void>>(async () => {}),
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
    get: vi.fn((_: Record<string, unknown>, cb: (items: Record<string, unknown>) => void) => {
      cb({})
    }),
    set: vi.fn((_: Record<string, unknown>, cb?: () => void) => cb && cb()),
  },
}

Object.defineProperty(globalThis, 'chrome', { value: { alarms, runtime, tabs, notifications, scripting, storage }, configurable: true })

// helper to reset between tests if needed
export function resetChromeMocks() {
  alarms.create.mockReset()
  alarms.onAlarm.addListener.mockReset?.()
  getURLMock.mockReset()
  runtime.onInstalled.addListener.mockReset?.()
  runtime.onMessage.addListener.mockReset?.()
  tabs.query.mockReset()
  tabs.sendMessage.mockReset()
  tabs.onUpdated.addListener.mockReset?.()
  notifications.create.mockReset()
  scripting.executeScript.mockReset()
  storage.local.get.mockReset()
  storage.local.set.mockReset()
  sendNativeMessageMock.mockReset()
  runtime.lastError = undefined
}

export function setChromeRuntimeLastError(err?: chrome.runtime.LastError) {
  runtime.lastError = err
}
