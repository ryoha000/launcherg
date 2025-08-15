import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleSyncDlsiteGames } from './syncDlsiteGames'

describe('dLsite 同期（syncDlsiteGames）ユースケース', () => {
  it('eGS 解決込みでネイティブに送信し、成功時は集計を記録して結果を返す', async () => {
    const egsInfo = create(EgsInfoSchema, {
      erogamescapeId: 345,
      gamename: 'Game B',
      gamenameRuby: 'げーむB',
      brandname: 'BrandB',
      brandnameRuby: 'ぶらんどB',
      sellday: '2024-02-02',
      isNukige: false,
    })
    const nativeResponse = {
      success: true,
      response: {
        case: 'syncGamesResult',
        value: {
          successCount: 2,
          errorCount: 0,
          errors: [],
          syncedGames: ['DL-1', 'DL-2'],
        },
      },
    }
    const record = vi.fn(async () => {})
    const send = vi.fn(async () => nativeResponse)
    const resolveForDlsite = vi.fn(async () => egsInfo)

    const context = buildTestContext({
      aggregation: { record },
      nativeMessenger: { send },
      egsResolver: { resolveForDlsite, resolveForDmm: async () => null },
      idGenerator: { generate: () => 'req-2' },
      extensionId: 'ext-id',
    })

    const req = {
      games: [
        { id: 'DL-1', category: 'gc' },
        { id: 'DL-2', category: 'gc' },
      ],
    } as any

    const res = await handleSyncDlsiteGames(context, 'rest-3', req)
    expect(send).toHaveBeenCalled()
    expect(resolveForDlsite).toHaveBeenCalledWith('DL-1', 'gc')
    expect(record).toHaveBeenCalledWith(2)
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
    expect((res.response as any).value.message).toContain('2個')
  })

  it('失敗時は success=false とエラーメッセージを返す', async () => {
    const send = vi.fn(async () => ({ success: false, error: 'fail' }))
    const context = buildTestContext({ nativeMessenger: { send } })
    const req = { games: [{ id: 'DL-X', category: 'gc' }] } as any
    const res = await handleSyncDlsiteGames(context, 'rest-4', req)
    expect(res.success).toBe(false)
    expect(res.error).toBe('fail')
  })
})
