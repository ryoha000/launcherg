import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleSyncDlsiteGames } from './syncDlsiteGames'

describe('dLsite 同期（syncDlsiteGames）ユースケース', () => {
  it('同期リクエストをプールに追加し、即時に成功レスポンスを返す', async () => {
    const add = vi.fn(() => {})
    const context = buildTestContext({
      syncPool: { add, sync: async () => {} },
    })

    const req = {
      games: [
        { id: 'DL-1', category: 'gc' },
        { id: 'DL-2', category: 'gc' },
      ],
    } as any

    const res = await handleSyncDlsiteGames(context, 'rest-3', req)

    expect(add).toHaveBeenCalledWith({ type: 'dlsite', games: req.games })
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
    expect((res.response as any).value.message).toContain('プールに追加しました')
  })
})
