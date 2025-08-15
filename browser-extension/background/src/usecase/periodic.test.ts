import { beforeEach, describe, expect, it, vi } from 'vitest'
import { resetChromeMocks } from '../../test/setup/chrome'
import { performPeriodicSync } from './periodic'

describe('定期同期（periodic）ユースケース', () => {
  beforeEach(() => {
    resetChromeMocks()
  })

  it('対象URLのタブにメッセージを送信する', async () => {
    chrome.tabs.query = vi.fn(async () => [
      { id: 1, url: 'https://dlsoft.dmm.co.jp/foo' },
      { id: 2, url: 'https://play.dlsite.com/bar' },
    ] as any)
    await performPeriodicSync()
    expect(chrome.tabs.sendMessage).toHaveBeenCalledTimes(2)
  })

  it('受信者不在なら注入して再送信する', async () => {
    chrome.tabs.query = vi.fn(async () => [{ id: 1, url: 'https://dlsoft.dmm.co.jp/x' }] as any)
    const sendMessageMock = vi.fn(async () => {
      throw new Error('Receiving end does not exist.')
    })
    chrome.tabs.sendMessage = sendMessageMock as any
    await performPeriodicSync()
    expect(chrome.scripting.executeScript).toHaveBeenCalled()
    expect(chrome.tabs.sendMessage).toHaveBeenCalledTimes(2)
  })

  it('その他のエラーでは注入せずにスキップする', async () => {
    chrome.tabs.query = vi.fn(async () => [{ id: 1, url: 'https://dlsoft.dmm.co.jp/x' }] as any)
    const sendMessageMock = vi.fn(async () => {
      throw new Error('some other error')
    })
    chrome.tabs.sendMessage = sendMessageMock as any
    await performPeriodicSync()
    expect(chrome.scripting.executeScript).not.toHaveBeenCalled()
  })
})
