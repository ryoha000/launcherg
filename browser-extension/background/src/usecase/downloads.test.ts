import type { DownloadIntentMap } from '@launcherg/shared'
import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { discardFailedDownloadIntent, readAllDownloadIntents, registerDownloadIntent, updateDownloadIntent } from '@launcherg/shared'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { setupDownloadsHandler, startIntentDownloads } from './downloads'

let storage: Record<string, DownloadIntentMap>
let items: chrome.downloads.DownloadItem[]
const startedAt = Date.now()
function item(id: number): chrome.downloads.DownloadItem {
  return { id, url: 'https://example.test/?workno=RJ1', filename: `part-${id}.zip`, state: 'complete', startTime: new Date(startedAt).toISOString(), mime: 'application/zip' } as chrome.downloads.DownloadItem
}
beforeEach(() => {
  storage = {}
  items = []
  vi.stubGlobal('chrome', {
    storage: { local: {
      get: vi.fn(async () => structuredClone(storage)),
      set: vi.fn(async (value: Record<string, DownloadIntentMap>) => { storage = structuredClone(value) }),
    } },
    downloads: { onChanged: { addListener: vi.fn() }, search: vi.fn(async (query: { id?: number }) => query.id ? items.filter(item => item.id === query.id) : items) },
    alarms: { create: vi.fn(), onAlarm: { addListener: vi.fn() } },
  })
})

describe('ダウンロード完了と復旧', () => {
  it('再開できない中断を一度だけ再ダウンロードし開始失敗も回収する', async () => {
    const retry = setupDownloadsHandler(buildTestContext())
    await registerDownloadIntent('RJ1', { store: 'DLsite', game: { storeId: 'RJ1' }, expected: 1, completed: 0, startedAt })
    const download = vi.fn().mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce(1).mockResolvedValueOnce(2)
    chrome.downloads.download = download
    await expect(startIntentDownloads('RJ1', ['https://example.test/?workno=RJ1'])).rejects.toThrow('開始に失敗')
    expect((await readAllDownloadIntents()).RJ1.pendingUrls).toHaveLength(1)
    await retry(true)
    items = [{ ...item(1), state: 'interrupted', canResume: false }]
    await retry(true)
    await retry(true)
    expect(download).toHaveBeenCalledTimes(3)
    expect((await readAllDownloadIntents()).RJ1.downloadIds).toEqual([2])
  })

  it('失敗した処理だけ解除してストアから開始し直せる', async () => {
    const entry = { store: 'DLsite' as const, game: { storeId: 'RJ1' }, expected: 1, completed: 0, startedAt }
    await registerDownloadIntent('RJ1', entry)
    await expect(discardFailedDownloadIntent('RJ1')).rejects.toThrow('失敗した処理だけ')
    await updateDownloadIntent('RJ1', { lastError: 'expired URL' })
    await discardFailedDownloadIntent('RJ1')
    await expect(registerDownloadIntent('RJ1', entry)).resolves.toBeUndefined()
  })
  it('分割の同時完了と重複通知を集約し失敗を保持して再試行する', async () => {
    const sendJson = vi.fn<() => Promise<NativeResponseTs>>().mockResolvedValue({ success: false, error: 'disk full', request_id: '' })
    const retry = setupDownloadsHandler(buildTestContext({ nativeMessenger: { sendJson } }))
    await registerDownloadIntent('RJ1', { store: 'DLsite', game: { storeId: 'RJ1' }, expected: 2, completed: 0, startedAt })
    items = [item(1), item(2)]
    const notify = (id: number) => (globalThis as ServiceWorkerGlobalScope).__launchergDownloadsOnChangedHandler!({ id, state: { current: 'complete' } })
    await Promise.all([notify(1), notify(2), notify(1)])
    const entry = (await readAllDownloadIntents()).RJ1
    expect(entry.completed).toBe(2)
    expect(entry.items).toHaveLength(2)
    expect(entry.lastError).toBe('disk full')
    expect(sendJson).toHaveBeenCalledTimes(1)
    const firstRequest = sendJson.mock.calls[0]
    sendJson.mockResolvedValue({ success: true, error: '', request_id: '' })
    await retry(true)
    expect((await readAllDownloadIntents()).RJ1).toBeUndefined()
    expect(sendJson.mock.calls[1]).toEqual(firstRequest)
  })

  it('停止中に届いた完了を起動時に回収する', async () => {
    await registerDownloadIntent('RJ1', { store: 'DLsite', game: { storeId: 'RJ1' }, expected: 1, completed: 0, startedAt })
    items = [item(1)]
    const sendJson = vi.fn(async () => ({ success: true, error: '', request_id: '' }))
    setupDownloadsHandler(buildTestContext({ nativeMessenger: { sendJson } }))
    await vi.waitFor(() => expect(sendJson).toHaveBeenCalledOnce())
    await vi.waitFor(async () => expect(await readAllDownloadIntents()).toEqual({}))
  })

  it('別作品の同時登録を失わず未完了作品の上書きを拒否する', async () => {
    const entry = (id: string) => ({ store: 'DLsite' as const, game: { storeId: id }, expected: 1, completed: 0, startedAt })
    await Promise.all([registerDownloadIntent('RJ1', entry('RJ1')), registerDownloadIntent('RJ2', entry('RJ2'))])
    expect(Object.keys(await readAllDownloadIntents()).sort()).toEqual(['RJ1', 'RJ2'])
    await expect(registerDownloadIntent('RJ1', entry('RJ1'))).rejects.toThrow('未完了')
  })
})
