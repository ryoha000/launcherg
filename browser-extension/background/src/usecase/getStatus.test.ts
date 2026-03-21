import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleGetStatus } from './getStatus'

describe('ステータス取得（getStatus）ユースケース', () => {
  it('ネイティブの getStatus を組み立てて StatusData にマップする', async () => {
    const context = buildTestContext({
      idGenerator: { generate: () => 'rid' },
      nativeMessenger: {
        sendJson: vi.fn(async (): Promise<NativeResponseTs> => ({
          success: true,
          error: '',
          request_id: 'rid',
          response: {
            case: 'StatusResult',
            value: {
              last_sync: { seconds: 1000, nanos: 0 },
              total_synced: 5,
              connected_extensions: 1,
              is_running: true,
              connection_status: 'connected',
              error_message: '',
            },
          },
        })),
      },
    })
    const res = await handleGetStatus(context, 'req-1', {})
    expect(context.nativeMessenger.sendJson).toHaveBeenCalled()
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('statusResult')
  })
})
