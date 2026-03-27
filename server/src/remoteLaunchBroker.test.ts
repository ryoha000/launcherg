import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('cloudflare:workers', () => ({
  DurableObject: class {},
}))

import { RemoteLaunchBrokerState } from '@server/remoteLaunchBroker'

function createDesktopSocket() {
  const sentMessages: string[] = []
  const socket = {
    send(message: string) {
      sentMessages.push(message)
    },
    deserializeAttachment() {
      return {
        kind: 'desktop',
        deviceId: 'device-1',
      }
    },
  } as unknown as WebSocket

  return {
    socket,
    sentMessages,
  }
}

describe('RemoteLaunchBrokerState', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('desktop 未接続時は即座に desktop_not_connected を返す', async () => {
    const state = new RemoteLaunchBrokerState()

    const response = await state.requestLaunch([], '123')

    expect(response).toEqual({
      ok: false,
      reasonCode: 'desktop_not_connected',
      message: 'デスクトップアプリが Remote Launch に接続していません。Launcherg を起動した状態で再試行してください。',
      detail: 'desktop websocket is not connected',
    })
    expect(state.getPendingLaunchCount()).toBe(0)
  })

  it('requestId ごとに ack を正しく対応付ける', async () => {
    const state = new RemoteLaunchBrokerState()
    const { socket, sentMessages } = createDesktopSocket()

    const first = state.requestLaunch([socket], 'work-a')
    const second = state.requestLaunch([socket], 'work-b')

    expect(sentMessages).toHaveLength(2)
    expect(state.getPendingLaunchCount()).toBe(2)

    const firstMessage = JSON.parse(sentMessages[0]) as { requestId: string }
    const secondMessage = JSON.parse(sentMessages[1]) as { requestId: string }

    state.handleWebSocketMessage(
      socket,
      JSON.stringify({
        type: 'launch-ack',
        requestId: secondMessage.requestId,
        ok: true,
        reasonCode: 'launched',
        message: 'デスクトップアプリでゲームを起動しました。',
        detail: null,
      }),
    )
    state.handleWebSocketMessage(
      socket,
      JSON.stringify({
        type: 'launch-ack',
        requestId: firstMessage.requestId,
        ok: false,
        reasonCode: 'shortcut_not_found',
        message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
        detail: 'no launchable lnk found',
      }),
    )

    await expect(second).resolves.toEqual({
      ok: true,
      reasonCode: 'launched',
      message: 'デスクトップアプリでゲームを起動しました。',
      detail: null,
    })
    await expect(first).resolves.toEqual({
      ok: false,
      reasonCode: 'shortcut_not_found',
      message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
      detail: 'no launchable lnk found',
    })
    expect(state.getPendingLaunchCount()).toBe(0)
  })

  it('ack が返らない場合は timeout で失敗する', async () => {
    vi.useFakeTimers()
    const state = new RemoteLaunchBrokerState()
    const { socket, sentMessages } = createDesktopSocket()

    const pending = state.requestLaunch([socket], 'work-timeout')
    expect(sentMessages).toHaveLength(1)
    expect(state.getPendingLaunchCount()).toBe(1)

    await vi.advanceTimersByTimeAsync(15_000)

    await expect(pending).resolves.toEqual({
      ok: false,
      reasonCode: 'timeout',
      message: 'デスクトップアプリから起動結果を受信できませんでした。しばらくしてから再試行してください。',
      detail: 'launch ack timed out',
    })
    expect(state.getPendingLaunchCount()).toBe(0)
    vi.useRealTimers()
  })
})
