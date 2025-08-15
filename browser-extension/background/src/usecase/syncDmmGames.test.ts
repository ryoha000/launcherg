import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleSyncDmmGames } from './syncDmmGames'

describe('dMM 同期（syncDmmGames）ユースケース', () => {
  it('eGS 解決込みでネイティブに送信し、成功時は集計を記録して結果を返す', async () => {
    const egsInfo = create(EgsInfoSchema, {
      erogamescapeId: 123,
      gamename: 'Game A',
      gamenameRuby: 'げーむA',
      brandname: 'Brand',
      brandnameRuby: 'ぶらんど',
      sellday: '2024-01-01',
      isNukige: false,
    })
    const nativeResponse = {
      success: true,
      response: {
        case: 'syncGamesResult',
        value: {
          successCount: 1,
          errorCount: 0,
          errors: [],
          syncedGames: ['ABC123'],
        },
      },
    }
    const record = vi.fn(async () => {})
    const send = vi.fn(async () => nativeResponse)
    const resolveForDmm = vi.fn(async () => egsInfo)

    const context = buildTestContext({
      aggregation: { record },
      nativeMessenger: { send },
      egsResolver: { resolveForDmm, resolveForDlsite: async () => null },
      idGenerator: { generate: () => 'req-1' },
      extensionId: 'ext-id',
    })

    const req = {
      games: [{ id: 'ABC123', category: 'cat', subcategory: 'sub' }],
    } as any

    const res = await handleSyncDmmGames(context, 'rest-1', req)
    expect(send).toHaveBeenCalled()
    expect(resolveForDmm).toHaveBeenCalledWith('ABC123', 'cat', 'sub')
    expect(record).toHaveBeenCalledWith(1)
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
    expect((res.response as any).value.message).toContain('1個')
  })

  it('失敗時は success=false とエラーメッセージを返す', async () => {
    const send = vi.fn(async () => ({ success: false, error: 'boom' }))
    const context = buildTestContext({ nativeMessenger: { send } })
    const req = { games: [{ id: 'X', category: 'c', subcategory: 's' }] } as any
    const res = await handleSyncDmmGames(context, 'rest-2', req)
    expect(res.success).toBe(false)
    expect(res.error).toBe('boom')
  })
})
