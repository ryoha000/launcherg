import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleSyncDlsiteGames } from './syncDlsiteGames'

describe('dLsite 同期（syncDlsiteGames）ユースケース', () => {
  it('同期リクエストを即時実行し、成功レスポンスを返す', async () => {
    const runExclusive = vi.fn(async callback => await callback())
    const context = buildTestContext({
      syncCoordinator: { runExclusive },
      egsResolver: {
        resolveForDmm: async () => null,
        resolveForDlsite: async () => null,
        resolveForDmmBulk: vi.fn(async items => items.map(() => null)),
        resolveForDlsiteBulk: vi.fn(async items => items.map(() => null)),
      },
      nativeMessenger: {
        sendJson: vi.fn(async () => ({
          success: true,
          error: '',
          request_id: 'native-1',
          response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 0, error_count: 0, errors: [], synced_games: [] } },
        } satisfies NativeResponseTs)),
      },
    })

    const req = {
      games: [
        { id: 'DL-1', category: 'gc' },
        { id: 'DL-2', category: 'gc' },
      ],
    } as any

    const res = await handleSyncDlsiteGames(context, 'rest-3', req)

    expect(runExclusive).toHaveBeenCalledTimes(1)
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
    expect((res.response as any).value.message).toContain('同期を実行しました')
  })
})
