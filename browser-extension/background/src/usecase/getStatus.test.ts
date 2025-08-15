import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleGetStatus } from './getStatus'

describe('ステータス取得（getStatus）ユースケース', () => {
  it('ネイティブの getStatus を組み立てて StatusData にマップする', async () => {
    const nativeResponse = {
      success: true,
      response: {
        case: 'statusResult',
        value: {
          lastSync: { seconds: BigInt(1000), nanos: 0 },
          totalSynced: 5,
          connectedExtensions: 1,
          isRunning: true,
          connectionStatus: 1,
          errorMessage: '',
        },
      },
    }
    const context = buildTestContext({
      idGenerator: { generate: () => 'rid' },
      nativeMessenger: { send: vi.fn(async () => nativeResponse) },
    })
    const res = await handleGetStatus(context, 'req-1', {} as any)
    expect((context.nativeMessenger as any).send).toHaveBeenCalled()
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('statusResult')
  })
})
