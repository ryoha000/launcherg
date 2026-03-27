import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

import { api } from '@ui/api'

describe('api.launchWork', () => {
  beforeEach(() => {
    vi.stubGlobal('window', {
      location: {
        origin: 'https://example.com',
      },
    })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('JSON の成功レスポンスを正しく parse する', async () => {
    const fetchMock = vi.fn(async () => Response.json({
      ok: true,
      reasonCode: 'launched',
      message: 'デスクトップアプリでゲームを起動しました。',
      detail: null,
    }))
    vi.stubGlobal('fetch', fetchMock)

    const response = await api.launchWork('device-1', 'work-1')

    expect(response).toEqual({
      ok: true,
      reasonCode: 'launched',
      message: 'デスクトップアプリでゲームを起動しました。',
      detail: null,
    })
    expect(fetchMock).toHaveBeenCalledWith(
      'https://example.com/api/device/device-1/works/work-1/launch',
      expect.objectContaining({
        method: 'POST',
        credentials: 'include',
      }),
    )
  })

  it('JSON の失敗レスポンスを正しく parse する', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => Response.json({
      ok: false,
      reasonCode: 'shortcut_not_found',
      message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
      detail: 'no launchable lnk found',
    })))

    const response = await api.launchWork('device-1', 'work-2')

    expect(response).toEqual({
      ok: false,
      reasonCode: 'shortcut_not_found',
      message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
      detail: 'no launchable lnk found',
    })
  })
})
