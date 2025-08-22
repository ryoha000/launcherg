import type { GetStatusRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import {
  ExtensionResponseSchema,
  GetStatusResponseSchema,
  StatusDataSchema,
} from '@launcherg/shared/proto/extension_internal'

export async function handleGetStatus(
  context: HandlerContext,
  requestId: string,
  _getStatusRequest: GetStatusRequest,
) {
  const nativeMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'GetStatus', value: {} },
  }
  const nativeResponse: any = await context.nativeMessenger.sendJson?.(nativeMessage)

  let statusData
  if (nativeResponse && nativeResponse.response?.case === 'StatusResult') {
    const syncStatus = nativeResponse.response.value
    statusData = create(StatusDataSchema, {
      lastSync: syncStatus.last_sync
        ? new Date(Number(syncStatus.last_sync.seconds) * 1000).toISOString()
        : '',
      totalSynced: Number(syncStatus.total_synced),
      connectedExtensions: Array.from({ length: syncStatus.connected_extensions || 0 }).map((_, i) => `ext-${i + 1}`),
      isRunning: syncStatus.is_running,
      connectionStatus: String(syncStatus.connection_status),
      errorMessage: syncStatus.error_message,
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
