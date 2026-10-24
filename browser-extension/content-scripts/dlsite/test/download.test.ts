import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { showInPageNotification } from '@launcherg/shared'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { findZipDownloadUrl, initLaunchergDownloadOnceForUrl, parseLaunchergParam } from '../src/download'

vi.mock('@launcherg/shared', async original => ({
  ...await original<typeof import('@launcherg/shared')>(),
  showInPageNotification: vi.fn(),
  reportIntegrationDiagnostic: vi.fn(),
}))

describe('dlsite download helpers', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
    vi.stubGlobal('chrome', { runtime: { sendMessage: vi.fn(async () => ({ success: true, startedCount: 1 })) } })
    document.body.innerHTML = ''
    // Reset URL (same-origin path only for JSDOM)
    window.history.replaceState({}, '', '/')
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('parseLaunchergParam: クエリが無ければ null', () => {
    const p = parseLaunchergParam()
    expect(p).toBeNull()
  })

  it('parseLaunchergParam: 正しい JSON を返す', () => {
    const payload = { type: 'download', value: { game: { storeId: 'RJ01234567' } } }
    const url = `/?launcherg=${encodeURIComponent(JSON.stringify(payload))}`
    window.history.pushState({}, '', url)

    const p = parseLaunchergParam()
    expect(p).not.toBeNull()
    expect(p?.type).toBe('download')
    expect(p?.value.game.storeId).toBe('RJ01234567')
  })

  it('旧画面のZIPリンクから対象作品のURLを取り出す', () => {
    const html = readFileSync(resolve(__dirname, 'data', 'dlsite_detail.html'), 'utf-8').replaceAll('https://play.dlsite.com', window.location.origin)
    const parser = new DOMParser()
    const doc = parser.parseFromString(html, 'text/html')
    expect(findZipDownloadUrl('RJ01363269', doc)).toContain('/api/v3/download?workno=RJ01363269')
  })

  function detailUrl(suffix = ''): string {
    const payload = { type: 'download', value: { game: { storeId: 'RJ-TEST' } } }
    return `/work/RJ-TEST${suffix}?launcherg=${encodeURIComponent(JSON.stringify(payload))}`
  }

  it('末尾にtreeがない現行URLで遅れて表示されるZIPリンクを待ち開始応答後に閉じる', async () => {
    window.history.replaceState({}, '', detailUrl())
    const running = initLaunchergDownloadOnceForUrl(window.location.href, vi.fn(), () => false)
    expect(chrome.runtime.sendMessage).not.toHaveBeenCalled()
    document.body.innerHTML = '<a href="/api/v3/download?workno=RJ-TEST" target="_blank">ZIPダウンロード</a>'
    await running
    expect(chrome.runtime.sendMessage).toHaveBeenCalledWith(expect.objectContaining({ type: 'start_dlsite_downloads', payload: { storeId: 'RJ-TEST', urls: [`${window.location.origin}/api/v3/download?workno=RJ-TEST`] } }))
    const types = vi.mocked(chrome.runtime.sendMessage).mock.calls.map(call => call[0].type)
    expect(types).toEqual(['register_download_intent', 'start_dlsite_downloads', 'close_current_tab'])
  })

  it('treeから現行URLへ正規化されても同じ作品を重複開始しない', async () => {
    const marked = new Set<string>()
    window.history.replaceState({}, '', detailUrl('/tree'))
    const first = initLaunchergDownloadOnceForUrl(window.location.href, key => marked.add(key), key => marked.has(key))
    window.history.replaceState({}, '', detailUrl())
    await initLaunchergDownloadOnceForUrl(window.location.href, key => marked.add(key), key => marked.has(key))
    document.body.innerHTML = '<a href="/api/v3/download?workno=RJ-TEST">ZIPダウンロード</a>'
    await first
    expect(vi.mocked(chrome.runtime.sendMessage).mock.calls.filter(call => call[0].type === 'start_dlsite_downloads')).toHaveLength(1)
  })

  it('別作品や別サイトのZIPリンクでは開始せずタイムアウトを表示する', async () => {
    window.history.replaceState({}, '', detailUrl())
    document.body.innerHTML = '<a href="/api/v3/download?workno=RJ-OTHER">ZIPダウンロード</a><a href="https://example.com/api/v3/download?workno=RJ-TEST">ZIPダウンロード</a>'
    const running = initLaunchergDownloadOnceForUrl(window.location.href, vi.fn(), () => false)
    await vi.advanceTimersByTimeAsync(30_000)
    await running
    expect(chrome.runtime.sendMessage).not.toHaveBeenCalled()
    expect(showInPageNotification).toHaveBeenCalledWith(expect.stringContaining('リンクが見つかりません'), 'error')
  })

  it('ブラウザがダウンロード開始を拒否した場合はエラーを表示しタブを閉じない', async () => {
    window.history.replaceState({}, '', detailUrl())
    document.body.innerHTML = '<a href="/api/v3/download?workno=RJ-TEST">ZIPダウンロード</a>'
    vi.mocked(chrome.runtime.sendMessage).mockResolvedValueOnce({ success: true }).mockResolvedValueOnce({ success: false, error: '開始できません' })
    await initLaunchergDownloadOnceForUrl(window.location.href, vi.fn(), () => false)
    expect(chrome.runtime.sendMessage).not.toHaveBeenCalledWith({ type: 'close_current_tab' })
    expect(showInPageNotification).toHaveBeenCalledWith('開始できません', 'error')
  })
})
