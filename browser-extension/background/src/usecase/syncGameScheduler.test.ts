import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { NativeResponseSchema, SyncBatchResultSchema } from '@launcherg/shared/proto/native_messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { syncGame } from './syncGameScheduler'

describe('ゲーム同期スケジューラ（syncGameScheduler）', () => {
  it('dMM と DLsite をそれぞれバルク解決し、Native へ送信し、集計を記録する', async () => {
    const items = [
      { type: 'dmm' as const, games: [
        { id: 'D1', category: 'mono', subcategory: 'pcgame' },
        { id: 'D2', category: 'digital', subcategory: 'doujin' },
      ] },
      { type: 'dlsite' as const, games: [
        { id: 'RJ123', category: 'maniax' },
        { id: 'RJ456', category: 'girls' },
      ] },
    ]

    const resolveForDmmBulk = vi.fn(async (arr: Array<{ storeId: string, category: string, subcategory: string }>) => arr.map((_v, i) => create(EgsInfoSchema, {
      erogamescapeId: 100 + i,
      gamename: `DMM-${i}`,
      gamenameRuby: `dmm-${i}`,
      sellday: '2020-01-01',
      isNukige: false,
      brandname: 'brand',
      brandnameRuby: 'brand-ruby',
    })))
    const resolveForDlsiteBulk = vi.fn(async (arr: Array<{ storeId: string, category: string }>) => arr.map((_v, i) => create(EgsInfoSchema, {
      erogamescapeId: 200 + i,
      gamename: `DLS-${i}`,
      gamenameRuby: `dls-${i}`,
      sellday: '2021-01-01',
      isNukige: true,
      brandname: 'brand2',
      brandnameRuby: 'brand2-ruby',
    })))

    const send = vi.fn(async (message: any) => {
      const isDmm = message.message.case === 'syncDmmGames'
      const count = isDmm ? message.message.value.games.length : message.message.value.games.length
      return create(NativeResponseSchema, {
        success: true,
        error: '',
        requestId: 'res-1',
        response: {
          case: 'syncGamesResult',
          value: create(SyncBatchResultSchema, { successCount: count, errorCount: 0, errors: [], syncedGames: [] }),
        },
      })
    })

    const record = vi.fn(async (_n: number) => {})

    const context = buildTestContext({
      syncPool: {
        add: () => {},
        sync: async (callback) => { await callback(items as any) },
      },
      egsResolver: {
        resolveForDmm: async () => null,
        resolveForDlsite: async () => null,
        resolveForDmmBulk,
        resolveForDlsiteBulk,
      },
      nativeMessenger: { send },
      aggregation: { record },
    })

    await syncGame(context)

    expect(resolveForDmmBulk).toHaveBeenCalledTimes(1)
    expect(resolveForDlsiteBulk).toHaveBeenCalledTimes(1)
    expect(send).toHaveBeenCalledTimes(2)
    expect(record).toHaveBeenCalledTimes(2)
    expect(record).toHaveBeenNthCalledWith(1, 2)
    expect(record).toHaveBeenNthCalledWith(2, 2)
  })

  it('1件でもバルク解決を呼び出す', async () => {
    const items = [
      { type: 'dmm' as const, games: [{ id: 'DX', category: 'mono', subcategory: 'pcgame' }] },
      { type: 'dlsite' as const, games: [{ id: 'RJ999', category: 'girls' }] },
    ]

    const resolveForDmmBulk = vi.fn(async () => ([create(EgsInfoSchema, {
      erogamescapeId: 1,
      gamename: 'g',
      gamenameRuby: 'gr',
      sellday: '2020',
      isNukige: false,
      brandname: 'b',
      brandnameRuby: 'br',
    })]))
    const resolveForDlsiteBulk = vi.fn(async () => ([create(EgsInfoSchema, {
      erogamescapeId: 2,
      gamename: 'g2',
      gamenameRuby: 'gr2',
      sellday: '2021',
      isNukige: true,
      brandname: 'b2',
      brandnameRuby: 'br2',
    })]))

    const send = vi.fn(async (_message: any) => create(NativeResponseSchema, {
      success: true,
      error: '',
      requestId: 'res-2',
      response: { case: 'syncGamesResult', value: create(SyncBatchResultSchema, { successCount: 1, errorCount: 0, errors: [], syncedGames: [] }) },
    }))

    const record = vi.fn(async (_n: number) => {})

    const context = buildTestContext({
      syncPool: {
        add: () => {},
        sync: async (callback) => { await callback(items as any) },
      },
      egsResolver: { resolveForDmm: async () => null, resolveForDlsite: async () => null, resolveForDmmBulk, resolveForDlsiteBulk },
      nativeMessenger: { send },
      aggregation: { record },
    })

    await syncGame(context)

    expect(resolveForDmmBulk).toHaveBeenCalledTimes(1)
    expect(resolveForDlsiteBulk).toHaveBeenCalledTimes(1)
    expect(send).toHaveBeenCalledTimes(2)
    expect(record).toHaveBeenCalledTimes(2)
  })
})
