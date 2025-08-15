import { beforeEach, describe, expect, it, vi } from 'vitest'
import { handleDebugNativeMessage } from '../debugNativeMessage'

beforeEach(() => {})

describe('handleDebugNativeMessage', () => {
  it('ネイティブの応答JSONを返す', async () => {
    const context = {
      nativeHostName: 'host',
      extensionId: 'test-extension-id',
      sendNativeProtobufMessage: vi.fn(async () => ({ pong: true })),
      generateRequestId: () => 'req-4',
      resolveEgsForDmm: vi.fn(),
      resolveEgsForDlsite: vi.fn(),
      recordSyncAggregation: vi.fn(async () => {}),
    }

    const res = await handleDebugNativeMessage(context as any, 'r-4', {} as any)
    expect(context.sendNativeProtobufMessage).toHaveBeenCalledOnce()
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('debugResult')
    const value = (res.response as any).value
    expect(value.nativeResponseJson).toContain('pong')
  })
})
