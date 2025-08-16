import type { Browser } from '../shared/types'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireAggregateNotification, recordSyncAggregation } from './aggregation'

describe('集計（aggregation）ユースケース', () => {
  let browser: Browser
  beforeEach(() => {
    browser = {
      alarms: { create: vi.fn(async () => {}) },
      notifications: { create: vi.fn(async () => {}) },
      runtime: { getURL: (p: string) => `chrome-extension://test/${p}` },
      storage: {
        get: vi.fn(async () => ({})),
        set: vi.fn(async () => {}),
      },
      tabs: { query: vi.fn(async () => []), sendMessage: vi.fn(async () => {}) },
      scripting: { executeScript: vi.fn(async () => {}) },
    }
  })

  it('recordSyncAggregation: 加算してアラームをスケジュールする', async () => {
    await recordSyncAggregation(browser, 2)
    expect(browser.alarms.create).toHaveBeenCalled()
    expect(browser.storage.set).toHaveBeenCalled()
  })

  it('fireAggregateNotification: 合計>0で通知しリセットする', async () => {
    ;(browser.storage.get as any) = vi.fn(async () => ({ aggregate_sync_count: 3 }))
    await fireAggregateNotification(browser)
    expect(browser.notifications.create).toHaveBeenCalled()
    expect(browser.storage.set).toHaveBeenCalledWith({ aggregate_sync_count: 0 })
  })

  it('fireAggregateNotification: 合計=0なら何もしない', async () => {
    ;(browser.storage.get as any) = vi.fn(async () => ({ aggregate_sync_count: 0 }))
    await fireAggregateNotification(browser)
    expect(browser.notifications.create).not.toHaveBeenCalled()
  })
})
