import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleSyncDmmGames } from './syncDmmGames'

describe('dMM 同期（syncDmmGames）ユースケース', () => {
  it('同期リクエストをプールに追加し、即時に成功レスポンスを返す', async () => {
    const add = vi.fn(() => {})
    const context = buildTestContext({
      syncPool: { add, sync: async () => {} },
    })

    const req = {
      games: [{ id: 'ABC123', category: 'cat', subcategory: 'sub' }],
    } as any

    const res = await handleSyncDmmGames(context, 'rest-1', req)

    expect(add).toHaveBeenCalledWith({ type: 'dmm', games: req.games })
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
    expect((res.response as any).value.message).toContain('プールに追加しました')
  })
})
