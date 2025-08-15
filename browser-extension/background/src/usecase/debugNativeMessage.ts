import type { DebugNativeMessageRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessage } from '@launcherg/shared/proto/native_messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import {
  DebugNativeMessageResponseSchema,
  ExtensionResponseSchema,
} from '@launcherg/shared/proto/extension_internal'
import {
  HealthCheckRequestSchema,
  NativeMessageSchema,
} from '@launcherg/shared/proto/native_messaging'

export async function handleDebugNativeMessage(
  context: HandlerContext,
  restRequestId: string,
  _debugRequest: DebugNativeMessageRequest,
) {
  const debugMessage = create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, {
      seconds: BigInt(Math.floor(Date.now() / 1000)),
    }),
    requestId: context.idGenerator.generate(),
    message: {
      case: 'healthCheck',
      value: create(HealthCheckRequestSchema, {}),
    },
  }) as NativeMessage

  const nativeResponse = await context.nativeMessenger.send(debugMessage)

  return create(ExtensionResponseSchema, {
    requestId: restRequestId,
    success: true,
    error: '',
    response: {
      case: 'debugResult',
      value: create(DebugNativeMessageResponseSchema, {
        nativeResponseJson: JSON.stringify(nativeResponse),
        timestamp: new Date().toISOString(),
      }),
    },
  })
}
