import type { NativeMessage, NativeResponse } from '@launcherg/shared/proto/native_messaging'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import { NativeResponseSchema, SyncStatusSchema } from '@launcherg/shared/proto/native_messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleGetStatus } from './getStatus'

describe('ステータス取得（getStatus）ユースケース', () => {
  it('ネイティブの getStatus を組み立てて StatusData にマップする', async () => {
    const nativeResponse: NativeResponse = create(NativeResponseSchema, {
      success: true,
      response: {
        case: 'statusResult',
        value: create(SyncStatusSchema, {
          lastSync: create(TimestampSchema, { seconds: BigInt(1000), nanos: 0 }),
          totalSynced: 5,
          connectedExtensions: ['ext-1'],
          isRunning: true,
          connectionStatus: 1,
          errorMessage: '',
        }),
      },
    })
    const context = buildTestContext({
      idGenerator: { generate: () => 'rid' },
      nativeMessenger: { send: vi.fn(async (_: NativeMessage) => nativeResponse) },
    })
    const res = await handleGetStatus(context, 'req-1', {} as any)
    expect((context.nativeMessenger as any).send).toHaveBeenCalled()
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('statusResult')
  })
})
