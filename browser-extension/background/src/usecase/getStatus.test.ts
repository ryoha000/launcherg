import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleGetStatus } from './getStatus'

describe('ステータス取得（getStatus）ユースケース', () => {
  it('実際のヘルスチェックが成功した場合だけ接続済みとして返す', async () => {
    const context = buildTestContext({
      idGenerator: { generate: () => 'rid' },
      nativeMessenger: {
        sendJson: vi.fn(async (): Promise<NativeResponseTs> => ({
          success: true,
          error: '',
          request_id: 'rid',
          response: {
            case: 'HealthCheckResult',
            value: { message: 'ok', version: '0.4.0' },
          },
        })),
      },
    })
    const res = await handleGetStatus(context, 'req-1', {})
    expect(context.nativeMessenger.sendJson).toHaveBeenCalledWith({ request_id: 'rid', message: { case: 'HealthCheck', value: {} } })
    expect(res.success).toBe(true)
    expect(res.response?.case).toBe('statusResult')
  })
  it.each([null, { success: false, error: 'Host failed', request_id: 'rid', response: undefined }])('失敗や応答なしを接続済みとして扱わない', async (response) => {
    const context = buildTestContext({ nativeMessenger: { sendJson: vi.fn(async () => response) } })
    await expect(handleGetStatus(context, 'req-1', {})).rejects.toThrow()
  })
})
