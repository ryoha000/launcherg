import type { DmmLibraryHookMessageData } from '../src/api'
import { describe, expect, it, vi } from 'vitest'
import { createDmmRuntime } from '../src/runtime'

function createHookEvent(payload: DmmLibraryHookMessageData): MessageEvent<unknown> {
  const event = new MessageEvent('message', { data: payload })
  Object.defineProperty(event, 'source', { value: window })
  return event
}

describe('dmm runtime', () => {
  it('payload 未取得時は再読み込みを促して失敗する', async () => {
    const runtime = createDmmRuntime({
      initialUrl: 'https://dlsoft.dmm.co.jp/mylibrary/',
      fetchPackParentMap: vi.fn(async () => new Map()),
      processPacks: vi.fn(async () => []),
      syncDmmGames: vi.fn(async () => {}),
      showErrorNotification: vi.fn(),
    })

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: false,
      message: 'DMM: APIレスポンス未取得。ページを再読み込みしてください',
      error: 'DMM: APIレスポンス未取得。ページを再読み込みしてください',
    })
  })

  it('hook で受けた payload を一度だけ同期する', async () => {
    const syncDmmGames = vi.fn(async () => {})
    const runtime = createDmmRuntime({
      initialUrl: 'https://dlsoft.dmm.co.jp/mylibrary/',
      fetchPackParentMap: vi.fn(async () => new Map()),
      processPacks: vi.fn(async () => []),
      syncDmmGames,
      showErrorNotification: vi.fn(),
    })
    const payload: DmmLibraryHookMessageData = {
      source: 'launcherg',
      type: 'launcherg:dmm-library-response',
      pageUrl: 'https://dlsoft.dmm.co.jp/mylibrary/',
      requestUrl: 'https://dlsoft.dmm.co.jp/ajax/v1/library?page=1',
      payload: {
        error: null,
        body: {
          library: [{
            contentId: 'ncpy_0007',
            productId: 'ncpy_0007',
            libraryProductType: 'single',
            floor: 'Apcgame',
            title: 'Monkeys!',
            packageImageUrl: 'https://pics.dmm.co.jp/digital/pcgame/ncpy_0007/ncpy_0007ps.jpg',
          }],
        },
      },
    }

    runtime.handleHookMessage(createHookEvent(payload))
    await vi.waitFor(() => {
      expect(syncDmmGames).toHaveBeenCalledTimes(1)
    })

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: true,
      message: 'DMM: 最新のAPIレスポンスは同期済みです',
    })
  })

  it('uRL変更時に cache をリセットする', async () => {
    const runtime = createDmmRuntime({
      initialUrl: 'https://dlsoft.dmm.co.jp/mylibrary/?page=1',
      fetchPackParentMap: vi.fn(async () => new Map()),
      processPacks: vi.fn(async () => []),
      syncDmmGames: vi.fn(async () => {}),
      showErrorNotification: vi.fn(),
    })

    runtime.handleHookMessage(createHookEvent({
      source: 'launcherg',
      type: 'launcherg:dmm-library-response',
      pageUrl: 'https://dlsoft.dmm.co.jp/mylibrary/?page=1',
      requestUrl: 'https://dlsoft.dmm.co.jp/ajax/v1/library?page=1',
      payload: { error: null, body: { library: [] } },
    }))

    runtime.handleUrlChange('https://dlsoft.dmm.co.jp/mylibrary/?page=2')

    await expect(runtime.syncLatest()).resolves.toEqual({
      success: false,
      message: 'DMM: APIレスポンス未取得。ページを再読み込みしてください',
      error: 'DMM: APIレスポンス未取得。ページを再読み込みしてください',
    })
  })
})
