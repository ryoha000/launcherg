import type { DebugNativeMessageRequest, ExtensionResponse } from '@launcherg/shared'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'

export async function handleDebugNativeMessage(
  context: HandlerContext,
  restRequestId: string,
  _debugRequest: DebugNativeMessageRequest,
): Promise<ExtensionResponse> {
  const debugMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'HealthCheck', value: {} },
  }
  const nativeResponse = await context.nativeMessenger.sendJson?.(debugMessage)

  return {
    requestId: restRequestId,
    success: true,
    error: '',
    response: {
      case: 'debugResult',
      value: {
        nativeResponseJson: JSON.stringify(nativeResponse ?? null),
        timestamp: new Date().toISOString(),
      },
    },
  }
}
