import type { ExtensionRequest } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { createMessageDispatcher } from './dispatcher'

describe('メッセージディスパッチャ', () => {
  let context: HandlerContext

  beforeEach(() => {
    context = buildTestContext({
      nativeMessenger: { sendJson: vi.fn(async () => ({ success: true, error: '', request_id: 'rid-1', response: { case: 'StatusResult', value: { total_synced: 0, connected_extensions: 1, is_running: true, connection_status: 'connected', error_message: '' } } })) },
      egsResolver: {
        resolveForDmm: vi.fn(async () => null),
        resolveForDlsite: vi.fn(async () => null),
        resolveForDmmBulk: vi.fn(async (items: Array<{ storeId: string, category: string, subcategory: string }>) => items.map(() => null)),
        resolveForDlsiteBulk: vi.fn(async (items: Array<{ storeId: string, category: string }>) => items.map(() => null)),
      },
      idGenerator: { generate: () => 'rid-1' },
    })
  })

  it('getStatus をルーティングし拡張レスポンスJSONを返す', async () => {
    const dispatcher = createMessageDispatcher(context)
    const request: ExtensionRequest = { requestId: 'ext-1', request: { case: 'getStatus', value: {} } }
    const result = await dispatcher(request) as any
    expect(result.success).toBe(true)
    expect(result.requestId).toBe('ext-1')
  })

  it('未知のリクエストタイプではエラーを返す', async () => {
    const dispatcher = createMessageDispatcher(context)
    const badJson = { requestId: 'x', request: { case: 'unknown', value: {} } }
    const result = await dispatcher(badJson) as any
    expect(result.success).toBe(false)
  })
})
