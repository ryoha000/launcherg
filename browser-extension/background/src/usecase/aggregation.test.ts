import { beforeEach, describe, expect, it, vi } from 'vitest'
import { resetChromeMocks } from '../../test/setup/chrome'
import { fireAggregateNotification, recordSyncAggregation } from './aggregation'

describe('集計（aggregation）ユースケース', () => {
  beforeEach(() => {
    resetChromeMocks()
  })

  it('recordSyncAggregation: 加算してアラームをスケジュールする', async () => {
    chrome.storage.local.get = vi.fn((_, cb) => cb({})) as any
    await recordSyncAggregation(2)
    expect(chrome.alarms.create).toHaveBeenCalled()
    expect(chrome.storage.local.set).toHaveBeenCalled()
  })

  it('fireAggregateNotification: 合計>0で通知しリセットする', async () => {
    chrome.storage.local.get = vi.fn((_, cb) => cb({ aggregate_sync_count: 3 })) as any
    await fireAggregateNotification()
    expect(chrome.notifications.create).toHaveBeenCalled()
    expect(chrome.storage.local.set).toHaveBeenCalledWith({ aggregate_sync_count: 0 }, expect.any(Function))
  })

  it('fireAggregateNotification: 合計=0なら何もしない', async () => {
    chrome.storage.local.get = vi.fn((_, cb) => cb({ aggregate_sync_count: 0 })) as any
    await fireAggregateNotification()
    expect(chrome.notifications.create).not.toHaveBeenCalled()
  })
})
