import { describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { handleDebugNativeMessage } from './debugNativeMessage'

describe('debug Native Message のユースケース', () => {
  it('healthCheck を送信し結果を debugResult でラップする', async () => {
    const context = buildTestContext({
      idGenerator: { generate: () => 'rid' },
      nativeMessenger: { send: vi.fn(async () => ({ success: true, response: { case: 'healthCheckResult', value: {} } })) },
    })
    const res = await handleDebugNativeMessage(context, 'req-1', {} as any)
    expect((context.nativeMessenger as any).send).toHaveBeenCalled()
    expect(res.requestId).toBe('req-1')
    expect(res.response?.case).toBe('debugResult')
  })
})
