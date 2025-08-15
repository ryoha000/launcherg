import { describe, expect, it, vi } from 'vitest'

// The module registers listeners at import-time. We mock dependencies first.
vi.mock('./usecase/aggregation', () => ({
  AGGREGATE_ALARM: 'notify_aggregate',
  fireAggregateNotification: vi.fn(),
  recordSyncAggregation: vi.fn(),
}))

vi.mock('./usecase/periodic', () => ({
  performPeriodicSync: vi.fn(),
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

    // import registers listeners
    await import('./background')
    expect(addAlarmListener).toHaveBeenCalled()
    expect(addMsgListener).toHaveBeenCalled()
    expect(addInstalled).toHaveBeenCalled()
    expect(addUpdated).toHaveBeenCalled()
  })
})
