import type { GetStatusRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessage } from '@launcherg/shared/proto/native_messaging'
import type { HandlerContext } from './handler'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import {
  ExtensionResponseSchema,

  GetStatusResponseSchema,
  StatusDataSchema,
} from '@launcherg/shared/proto/extension_internal'
import {
  GetStatusRequestSchema as NativeGetStatusRequestSchema,

  NativeMessageSchema,
} from '@launcherg/shared/proto/native_messaging'

export async function handleGetStatus(
  context: HandlerContext,
  requestId: string,
  _getStatusRequest: GetStatusRequest,
) {
  const nativeMessage = create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, {
      seconds: BigInt(Math.floor(Date.now() / 1000)),
    }),
    requestId: context.generateRequestId(),
    message: {
      case: 'getStatus',
      value: create(NativeGetStatusRequestSchema, {}),
    },
  }) as NativeMessage

  const nativeResponse = await context.sendNativeProtobufMessage(
    context.nativeHostName,
    nativeMessage,
  )

  let statusData
  if (nativeResponse && nativeResponse.response.case === 'statusResult') {
    const syncStatus = nativeResponse.response.value
    statusData = create(StatusDataSchema, {
      lastSync: syncStatus.lastSync
        ? new Date(Number(syncStatus.lastSync.seconds) * 1000).toISOString()
        : '',
      totalSynced: Number(syncStatus.totalSynced),
      connectedExtensions: syncStatus.connectedExtensions,
      isRunning: syncStatus.isRunning,
      connectionStatus: syncStatus.connectionStatus.toString(),
      errorMessage: syncStatus.errorMessage,
    })
  }

  return create(ExtensionResponseSchema, {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'statusResult',
      value: create(GetStatusResponseSchema, {
        status: statusData,
      }),
    },
  })
}
