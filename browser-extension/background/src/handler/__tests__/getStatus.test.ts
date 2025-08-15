import { beforeEach, describe, expect, it, vi } from 'vitest'
import { handleGetStatus } from '../getStatus'

beforeEach(() => {})

describe('handleGetStatus', () => {
  it('ネイティブのstatusResultを整形して返す', async () => {
    const context = {
      nativeHostName: 'host',
      sendNativeProtobufMessage: vi.fn(async () => ({
        response: {
          case: 'statusResult',
          value: {
            lastSync: { seconds: BigInt(1700000000) },
            totalSynced: 10,
            connectedExtensions: 1,
            isRunning: true,
            connectionStatus: { toString: () => 'CONNECTED' },
            errorMessage: '',
          },
        },
      })),
      generateRequestId: () => 'req-3',
      extensionId: 'test-extension-id',
      resolveEgsForDmm: vi.fn(),
      resolveEgsForDlsite: vi.fn(),
      recordSyncAggregation: vi.fn(async () => {}),
    }

    const res = await handleGetStatus(context as any, 'r-3', {} as any)

    expect(context.sendNativeProtobufMessage).toHaveBeenCalledOnce()
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('statusResult')
    const status = (res.response as any).value.status
    expect(status.isRunning).toBe(true)
    expect(status.connectionStatus).toBe('CONNECTED')
  })
})
