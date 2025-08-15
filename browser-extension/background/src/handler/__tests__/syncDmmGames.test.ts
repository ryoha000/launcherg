import { beforeEach, describe, expect, it, vi } from 'vitest'
import { handleSyncDmmGames } from '../syncDmmGames'

// chrome.runtime.id を参照するための最低限のスタブ
beforeEach(() => {})

describe('handleSyncDmmGames', () => {
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
            value: { successCount: 1, errorCount: 0, errors: [], syncedGames: [] },
          },
        }
      }),
      generateRequestId: () => 'req-1',
      resolveEgsForDmm: vi.fn(async () => ({
        erogamescapeId: 123,
        gamename: 'Game',
        gamenameRuby: 'げーむ',
        brandname: 'Brand',
        brandnameRuby: 'ぶらんど',
        sellday: '2020-01-01',
        isNukige: false,
      })),
      resolveEgsForDlsite: vi.fn(),
      recordSyncAggregation: vi.fn(async () => {}),
    }

    const res = await handleSyncDmmGames(context as any, 'r-1', {
      games: [{ id: 'DMM-1', category: 'pc', subcategory: 'game' }],
    } as any)

    expect(context.resolveEgsForDmm).toHaveBeenCalledOnce()
    expect(context.sendNativeProtobufMessage).toHaveBeenCalledOnce()
    expect(context.recordSyncAggregation).toHaveBeenCalledWith(1)

    // ネイティブ送信メッセージに EGS 情報が含まれているか
    const nativeMsg = sentMessages[0]
    expect(nativeMsg.message.case).toBe('syncDmmGames')
    const firstGame = (nativeMsg.message.value.games as any[])[0]
    expect(firstGame.egsInfo.gamename).toBe('Game')

    // 応答の整形
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('syncGamesResult')
  })
})
