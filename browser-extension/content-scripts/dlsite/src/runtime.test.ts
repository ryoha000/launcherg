import type { DlsiteWorksHookMessageData } from './api'
import { describe, expect, it, vi } from 'vitest'
import { createDlsiteRuntime } from './runtime'

function createHookEvent(payload: DlsiteWorksHookMessageData): MessageEvent<unknown> {
  const event = new MessageEvent('message', { data: payload })
  Object.defineProperty(event, 'source', { value: window })
  return event
}

describe('dlsite runtime', () => {
  it('payload 未取得時は再読み込みを促して失敗する', async () => {
    const runtime = createDlsiteRuntime({
      initialUrl: 'https://play.dlsite.com/library',
      processGames: games => games,
      syncDlsiteGames: vi.fn(async () => {}),
      showErrorNotification: vi.fn(),
    })

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: false,
      message: 'DLsite: APIレスポンス未取得。ページを再読み込みしてください',
      error: 'DLsite: APIレスポンス未取得。ページを再読み込みしてください',
    })
  })

  it('hook で受けた payload を一度だけ同期する', async () => {
    const syncDlsiteGames = vi.fn(async () => {})
    const runtime = createDlsiteRuntime({
      initialUrl: 'https://play.dlsite.com/library',
      processGames: games => games,
      syncDlsiteGames,
      showErrorNotification: vi.fn(),
    })

    runtime.handleHookMessage(createHookEvent({
      source: 'launcherg',
      type: 'launcherg:dlsite-works-response',
      pageUrl: 'https://play.dlsite.com/library',
      requestUrl: 'https://play.dlsite.com/api/v3/content/works',
      payload: {
        works: [{
          workno: 'RJ01007737',
          site_id: 'maniax',
          name: { ja_JP: 'クロアプスクランブル' },
          work_files: {
            main: 'https://img.dlsite.jp/modpub/images2/work/doujin/RJ01008000/RJ01007737_img_main.jpg',
          },
        }],
      },
    }))

    await vi.waitFor(() => {
      expect(syncDlsiteGames).toHaveBeenCalledTimes(1)
    })

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: true,
      message: 'DLsite: 最新のAPIレスポンスは同期済みです',
    })
  })

  it('uRL変更時に cache をリセットする', async () => {
    const runtime = createDlsiteRuntime({
      initialUrl: 'https://play.dlsite.com/library?page=1',
      processGames: games => games,
      syncDlsiteGames: vi.fn(async () => {}),
      showErrorNotification: vi.fn(),
    })

    runtime.handleHookMessage(createHookEvent({
      source: 'launcherg',
      type: 'launcherg:dlsite-works-response',
      pageUrl: 'https://play.dlsite.com/library?page=1',
      requestUrl: 'https://play.dlsite.com/api/v3/content/works?page=1',
      payload: { works: [] },
    }))
    runtime.handleUrlChange('https://play.dlsite.com/library?page=2')

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: false,
      message: 'DLsite: APIレスポンス未取得。ページを再読み込みしてください',
      error: 'DLsite: APIレスポンス未取得。ページを再読み込みしてください',
    })
  })
})
