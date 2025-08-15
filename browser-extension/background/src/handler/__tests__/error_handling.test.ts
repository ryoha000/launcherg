import { create, toJson } from '@bufbuild/protobuf'
import { ExtensionRequestSchema } from '@launcherg/shared/proto/extension_internal'
import { describe, expect, it, vi } from 'vitest'
import { createMessageHandler } from '../handler'

describe('createMessageHandler - error handling', () => {
  it('fromJson 失敗時にエラーレスポンスを返す', async () => {
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
    // わざと壊れたメッセージ（スキーマ不一致）
    const res = await handler({ not: 'match' })
    expect(res.error).toBeTypeOf('string')
    expect(res.success).not.toBe(true)
  })

  it('syncDmmGames でネイティブがエラー返却した場合は失敗を返す', async () => {
    const context = {
      nativeHostName: 'host',
      extensionId: 'test-extension-id',
      sendNativeProtobufMessage: vi.fn(async () => ({ success: false, error: 'ng' })),
      generateRequestId: () => 'req-1',
      resolveEgsForDmm: vi.fn(async () => null),
      resolveEgsForDlsite: vi.fn(),
      recordSyncAggregation: vi.fn(),
    }

    const msg = create(ExtensionRequestSchema, {
      requestId: 'r-1',
      request: { case: 'syncDmmGames', value: { games: [{ id: 'x', category: 'pc', subcategory: 'game' }] } },
    })
    const handler = createMessageHandler(context as any)
    const res = await handler(toJson(ExtensionRequestSchema, msg))

    expect(res.success).not.toBe(true)
    // 失敗時は response.case は undefined のため、エラーのみ検証
    expect(res.response?.case).toBeUndefined()
  })
})
