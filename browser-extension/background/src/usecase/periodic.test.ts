import type { Browser } from '../shared/types'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { performPeriodicSync } from './periodic'

describe('定期同期（periodic）ユースケース', () => {
  let browser: Browser
  beforeEach(() => {
    browser = {
      alarms: { create: async () => {} },
      notifications: { create: async () => {} },
      runtime: { getURL: (p: string) => `chrome-extension://test/${p}` },
      storage: { get: async () => ({}), set: async () => {} },
      tabs: { query: vi.fn(async () => []), sendMessage: vi.fn(async () => {}) },
      scripting: { executeScript: vi.fn(async () => {}) },
    }
  })

  it('対象URLのタブにメッセージを送信する', async () => {
    const queryMock = vi.fn(async () => [
      { id: 1, url: 'https://dlsoft.dmm.co.jp/foo' },
      { id: 2, url: 'https://play.dlsite.com/bar' },
    ])
    ;(browser.tabs as any).query = queryMock
    await performPeriodicSync(browser)
    expect(browser.tabs.sendMessage).toHaveBeenCalledTimes(2)
  })

  it('受信者不在なら注入して再送信する', async () => {
    const queryMock = vi.fn(async () => [{ id: 1, url: 'https://dlsoft.dmm.co.jp/x' }])
    ;(browser.tabs as any).query = queryMock
    const sendMock = vi.fn(async () => {
      throw new Error('Receiving end does not exist.')
    })
    ;(browser.tabs as any).sendMessage = sendMock
    await performPeriodicSync(browser)
    expect(browser.scripting.executeScript).toHaveBeenCalled()
    expect(browser.tabs.sendMessage).toHaveBeenCalledTimes(2)
  })

  it('その他のエラーでは注入せずにスキップする', async () => {
    const queryMock = vi.fn(async () => [{ id: 1, url: 'https://dlsoft.dmm.co.jp/x' }])
    ;(browser.tabs as any).query = queryMock
    const sendMock = vi.fn(async () => {
      throw new Error('some other error')
    })
    ;(browser.tabs as any).sendMessage = sendMock
    await performPeriodicSync(browser)
    expect(browser.scripting.executeScript).not.toHaveBeenCalled()
  })
})
