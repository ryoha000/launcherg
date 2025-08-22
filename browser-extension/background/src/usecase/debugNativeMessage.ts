import type { DebugNativeMessageRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import {
  DebugNativeMessageResponseSchema,
  ExtensionResponseSchema,
} from '@launcherg/shared/proto/extension_internal'

export async function handleDebugNativeMessage(
  context: HandlerContext,
  restRequestId: string,
  _debugRequest: DebugNativeMessageRequest,
) {
  const debugMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'HealthCheck', value: {} },
  }
  const nativeResponse = await context.nativeMessenger.sendJson?.(debugMessage)

  return create(ExtensionResponseSchema, {
    requestId: restRequestId,
    success: true,
    error: '',
    response: {
      case: 'debugResult',
      value: create(DebugNativeMessageResponseSchema, {
        nativeResponseJson: JSON.stringify(nativeResponse ?? null),
        timestamp: new Date().toISOString(),
      }),
    },
  })
}
