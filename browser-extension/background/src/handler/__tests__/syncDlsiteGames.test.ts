import { beforeEach, describe, expect, it, vi } from 'vitest'
import { handleSyncDlsiteGames } from '../syncDlsiteGames'

beforeEach(() => {})

describe('handleSyncDlsiteGames', () => {
  it('eGS情報を解決してネイティブへ送信し、成功結果を返す', async () => {
    const sentMessages: any[] = []
    const context = {
      nativeHostName: 'host',
      extensionId: 'test-extension-id',
      sendNativeProtobufMessage: vi.fn(async (_host, message) => {
        sentMessages.push(message)
        return {
          success: true,
          response: {
            case: 'syncGamesResult',
            value: { successCount: 2, errorCount: 0, errors: [], syncedGames: [] },
          },
        }
      }),
      generateRequestId: () => 'req-1',
      resolveEgsForDlsite: vi.fn(async () => ({
        erogamescapeId: 321,
        gamename: 'Work',
        gamenameRuby: 'わーく',
        brandname: 'Circle',
        brandnameRuby: 'さーくる',
        sellday: '2021-01-01',
        isNukige: true,
      })),
      resolveEgsForDmm: vi.fn(),
      recordSyncAggregation: vi.fn(async () => {}),
    }

    const res = await handleSyncDlsiteGames(context as any, 'r-2', {
      games: [{ id: 'DL-1', category: 'home' }],
    } as any)

    expect(context.resolveEgsForDlsite).toHaveBeenCalledOnce()
    expect(context.sendNativeProtobufMessage).toHaveBeenCalledOnce()
    expect(context.recordSyncAggregation).toHaveBeenCalledWith(2)

    const nativeMsg = sentMessages[0]
    expect(nativeMsg.message.case).toBe('syncDlsiteGames')
    const firstGame = (nativeMsg.message.value.games as any[])[0]
    expect(firstGame.egsInfo.gamename).toBe('Work')

    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
  })
})
