import { describe, expect, it, vi } from 'vitest'

vi.mock('./usecase/periodic', () => ({
  performPeriodicSync: vi.fn(),
}))

vi.mock('./usecase/integrationDiagnostics', () => ({
  DIAGNOSTICS_RETRY_ALARM: 'integration_diagnostics_retry',
  createIntegrationDiagnostics: () => ({ record: vi.fn(async () => {}), flush: vi.fn(async () => {}) }),
}))

vi.mock('./adapter/native/send', () => ({
  createNativeMessenger: () => ({ send: vi.fn(async () => null) }),
}))

vi.mock('./adapter/egs/resolver', () => ({
  createEgsResolver: () => ({
    resolveForDmm: vi.fn(async () => null),
    resolveForDlsite: vi.fn(async () => null),
  }),
}))

describe('バックグラウンドの配線', () => {
  it('アラームとメッセージのリスナーを登録する', async () => {
    const addAlarmListener = chrome.alarms.onAlarm.addListener
    const addMsgListener = chrome.runtime.onMessage.addListener
    const addInstalled = chrome.runtime.onInstalled.addListener
    const addUpdated = chrome.tabs.onUpdated.addListener

    chrome.runtime.getManifest = vi.fn(() => ({ manifest_version: 3 as const, name: 'test', version: '0.4.0' }))
    // import registers listeners
    await import('./background')
    expect(addAlarmListener).toHaveBeenCalled()
    expect(addMsgListener).toHaveBeenCalled()
    expect(addInstalled).toHaveBeenCalled()
    expect(addUpdated).toHaveBeenCalled()
  })
})
