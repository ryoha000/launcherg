import type { SyncCoordinator } from '../shared/types'
import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { syncGame } from './syncGameScheduler'

function createQueuedCoordinator(): SyncCoordinator {
  let tail = Promise.resolve<void>(undefined)

  return {
    async runExclusive<T>(callback: () => Promise<T>): Promise<T> {
      const current = tail.catch(() => undefined).then(callback)
      tail = current.then(() => undefined, () => undefined)
      return await current
    },
  }
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

async function waitForAssertion(assertion: () => void, attempts = 20): Promise<void> {
  let lastError: unknown
  for (let i = 0; i < attempts; i += 1) {
    try {
      assertion()
      return
    }
    catch (error) {
      lastError = error
      await Promise.resolve()
    }
  }
  throw lastError
}

describe('ゲーム同期ユースケース（syncGameScheduler）', () => {
  it('DMM 要求をバルク解決して Native へ送信する', async () => {
    const resolveForDmmBulk = vi.fn(async (arr: Array<{ storeId: string, category: string, subcategory: string }>) => arr.map((_v, i) => ({
      erogamescapeId: 100 + i,
      gamename: `DMM-${i}`,
      gamenameRuby: `dmm-${i}`,
      sellday: '2020-01-01',
      isNukige: false,
      brandname: 'brand',
      brandnameRuby: 'brand-ruby',
    })))
    const sendJson = vi.fn(async (_message: any) => ({
      success: true,
      error: '',
      request_id: 'res-1',
      response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 1, error_count: 0, errors: [], synced_games: [] } },
    } satisfies NativeResponseTs))

    const context = buildTestContext({
      egsResolver: {
        resolveForDmm: async () => null,
        resolveForDlsite: async () => null,
        resolveForDmmBulk,
        resolveForDlsiteBulk: vi.fn(async items => items.map(() => null)),
      },
      nativeMessenger: { sendJson },
    })

    await syncGame(context, {
      type: 'dmm',
      games: [
        { id: 'D1', category: 'mono', subcategory: 'pcgame' },
        { id: 'D2', category: 'digital', subcategory: 'doujin' },
      ] as any,
    })

    expect(resolveForDmmBulk).toHaveBeenCalledTimes(1)
    expect(sendJson).toHaveBeenCalledTimes(1)
  })

  it('DLsite 要求を1件でもバルク解決して Native へ送信する', async () => {
    const resolveForDlsiteBulk = vi.fn(async () => ([{
      erogamescapeId: 2,
      gamename: 'g2',
      gamenameRuby: 'gr2',
      sellday: '2021',
      isNukige: true,
      brandname: 'b2',
      brandnameRuby: 'br2',
    }]))
    const sendJson = vi.fn(async (_message: any) => ({
      success: true,
      error: '',
      request_id: 'res-2',
      response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 1, error_count: 0, errors: [], synced_games: [] } },
    } satisfies NativeResponseTs))

    const context = buildTestContext({
      egsResolver: {
        resolveForDmm: async () => null,
        resolveForDlsite: async () => null,
        resolveForDmmBulk: vi.fn(async items => items.map(() => null)),
        resolveForDlsiteBulk,
      },
      nativeMessenger: { sendJson },
    })

    await syncGame(context, {
      type: 'dlsite',
      games: [{ id: 'RJ999', category: 'girls' }] as any,
    })

    expect(resolveForDlsiteBulk).toHaveBeenCalledTimes(1)
    expect(sendJson).toHaveBeenCalledTimes(1)
  })

  it('new_count が 0 の場合は通知しない', async () => {
    const notificationsCreate = vi.fn(async () => {})
    const sendJson = vi.fn(async (_message: any) => ({
      success: true,
      error: '',
      request_id: 'res-3',
      response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 0, error_count: 0, errors: [], synced_games: [] } },
    } satisfies NativeResponseTs))

    const context = buildTestContext({
      nativeMessenger: { sendJson },
      browser: {
        ...buildTestContext().browser,
        notifications: { create: notificationsCreate },
      },
    })

    await syncGame(context, {
      type: 'dmm',
      games: [{ id: 'D1', category: 'mono', subcategory: 'pcgame' }] as any,
    })

    expect(sendJson).toHaveBeenCalledTimes(1)
    expect(notificationsCreate).not.toHaveBeenCalled()
  })

  it('new_count が 1 以上の場合だけ通知する', async () => {
    const notificationsCreate = vi.fn(async () => {})
    const sendJson = vi.fn(async (_message: any) => ({
      success: true,
      error: '',
      request_id: 'res-4',
      response: { case: 'SyncGamesResult', value: { success_count: 2, new_count: 2, error_count: 0, errors: [], synced_games: [] } },
    } satisfies NativeResponseTs))

    const context = buildTestContext({
      nativeMessenger: { sendJson },
      browser: {
        ...buildTestContext().browser,
        notifications: { create: notificationsCreate },
      },
    })

    await syncGame(context, {
      type: 'dlsite',
      games: [{ id: 'RJ1', category: 'maniax' }] as any,
    })

    expect(sendJson).toHaveBeenCalledTimes(1)
    expect(notificationsCreate).toHaveBeenCalledTimes(1)
  })

  it('同時要求はグローバルロックで直列化される', async () => {
    const firstGate = deferred<void>()
    const secondGate = deferred<void>()
    const order: string[] = []
    const sendJson = vi
      .fn()
      .mockImplementationOnce(async () => {
        order.push('first:start')
        await firstGate.promise
        order.push('first:end')
        return {
          success: true,
          error: '',
          request_id: 'res-5',
          response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 0, error_count: 0, errors: [], synced_games: [] } },
        } satisfies NativeResponseTs
      })
      .mockImplementationOnce(async () => {
        order.push('second:start')
        await secondGate.promise
        order.push('second:end')
        return {
          success: true,
          error: '',
          request_id: 'res-6',
          response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 0, error_count: 0, errors: [], synced_games: [] } },
        } satisfies NativeResponseTs
      })

    const context = buildTestContext({
      syncCoordinator: createQueuedCoordinator(),
      nativeMessenger: { sendJson },
    })

    const first = syncGame(context, {
      type: 'dmm',
      games: [{ id: 'D1', category: 'mono', subcategory: 'pcgame' }] as any,
    })
    const second = syncGame(context, {
      type: 'dlsite',
      games: [{ id: 'RJ1', category: 'maniax' }] as any,
    })

    await waitForAssertion(() => {
      expect(order).toEqual(['first:start'])
    })

    firstGate.resolve()
    await first
    await waitForAssertion(() => {
      expect(order).toEqual(['first:start', 'first:end', 'second:start'])
    })

    secondGate.resolve()
    await second
    expect(order).toEqual(['first:start', 'first:end', 'second:start', 'second:end'])
  })

  it('先行要求が失敗してもロックを解放して後続要求を処理する', async () => {
    const sendJson = vi
      .fn()
      .mockRejectedValueOnce(new Error('native failed'))
      .mockResolvedValueOnce({
        success: true,
        error: '',
        request_id: 'res-7',
        response: { case: 'SyncGamesResult', value: { success_count: 1, new_count: 0, error_count: 0, errors: [], synced_games: [] } },
      } satisfies NativeResponseTs)

    const context = buildTestContext({
      syncCoordinator: createQueuedCoordinator(),
      nativeMessenger: { sendJson },
    })

    await expect(syncGame(context, {
      type: 'dmm',
      games: [{ id: 'D1', category: 'mono', subcategory: 'pcgame' }] as any,
    })).rejects.toThrow('native failed')

    await expect(syncGame(context, {
      type: 'dlsite',
      games: [{ id: 'RJ1', category: 'maniax' }] as any,
    })).resolves.toBeUndefined()

    expect(sendJson).toHaveBeenCalledTimes(2)
  })
})
