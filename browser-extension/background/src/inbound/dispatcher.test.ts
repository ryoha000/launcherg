import type { HandlerContext } from '../shared/types'
import { create, fromJson, toJson } from '@bufbuild/protobuf'
import { ExtensionRequestSchema, ExtensionResponseSchema, GetStatusRequestSchema } from '@launcherg/shared/proto/extension_internal'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { createMessageDispatcher } from './dispatcher'

describe('メッセージディスパッチャ', () => {
  let context: HandlerContext

  beforeEach(() => {
    context = buildTestContext({
      nativeMessenger: { send: vi.fn(async () => ({
        success: true,
        response: {
          case: 'statusResult',
          value: {
            lastSync: undefined,
            totalSynced: 0,
            connectedExtensions: 1,
            isRunning: true,
            connectionStatus: 1,
            errorMessage: '',
          },
        },
      })) },
      egsResolver: {
        resolveForDmm: vi.fn(async () => null),
        resolveForDlsite: vi.fn(async () => null),
      },
      idGenerator: { generate: () => 'rid-1' },
    })
  })

  it('getStatus をルーティングし拡張レスポンスJSONを返す', async () => {
    const dispatcher = createMessageDispatcher(context)
    const request = create(ExtensionRequestSchema, {
      requestId: 'ext-1',
      request: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })
    const resultJson = await dispatcher(toJson(ExtensionRequestSchema, request))
    const result = fromJson(ExtensionResponseSchema, resultJson)
    expect(result.success).toBe(true)
    expect(result.requestId).toBe('ext-1')
  })

  it('未知のリクエストタイプではエラーを返す', async () => {
    const dispatcher = createMessageDispatcher(context)
    const badJson = { requestId: 'x', request: { case: 'unknown', value: {} } }
    const resultJson = await dispatcher(badJson)
    const result = fromJson(ExtensionResponseSchema, resultJson)
    expect(result.success).toBe(false)
  })
})
