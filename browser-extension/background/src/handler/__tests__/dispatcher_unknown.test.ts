import { describe, expect, it, vi } from 'vitest'
import { createMessageHandler } from '../handler'

describe('createMessageHandler - unknown request', () => {
  it('未知のリクエストタイプはエラーを返す', async () => {
    const context = {
      nativeHostName: 'host',
      extensionId: 'test-extension-id',
      sendNativeProtobufMessage: vi.fn(),
      generateRequestId: () => 'req-x',
      resolveEgsForDmm: vi.fn(),
      resolveEgsForDlsite: vi.fn(),
      recordSyncAggregation: vi.fn(),
    }

    const handler = createMessageHandler(context as any)
    // スキーマを通さずに未知の構造を渡す（受理時に default ブランチへ）
    const res = await handler({ requestId: 'r-u', request: { case: 'unknown', value: {} } })
    expect(String(res.error)).toContain('cannot decode message')
    expect(res.success).not.toBe(true)
  })
})
