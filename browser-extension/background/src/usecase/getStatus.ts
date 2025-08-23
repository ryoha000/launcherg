import type { ExtensionResponse, GetStatusRequest, StatusData } from '@launcherg/shared'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'

export async function handleGetStatus(
  context: HandlerContext,
  requestId: string,
  _getStatusRequest: GetStatusRequest,
): Promise<ExtensionResponse> {
  const nativeMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'GetStatus', value: {} },
  }
  const nativeResponse: any = await context.nativeMessenger.sendJson?.(nativeMessage)

  let statusData: StatusData | undefined
  if (nativeResponse && nativeResponse.response?.case === 'StatusResult') {
    const syncStatus = nativeResponse.response.value
    statusData = {
      lastSync: syncStatus.last_sync
        ? new Date(Number(syncStatus.last_sync.seconds) * 1000).toISOString()
        : '',
      totalSynced: Number(syncStatus.total_synced),
      connectedExtensions: Array.from({ length: syncStatus.connected_extensions || 0 }).map((_, i) => `ext-${i + 1}`),
      isRunning: syncStatus.is_running,
      connectionStatus: String(syncStatus.connection_status),
      errorMessage: syncStatus.error_message,
    }
  }

  return {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'statusResult',
      value: { status: statusData },
    },
  }
}
