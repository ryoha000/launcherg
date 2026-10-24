import type { ExtensionResponse, GetStatusRequest } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'

export async function handleGetStatus(context: HandlerContext, requestId: string, _request: GetStatusRequest): Promise<ExtensionResponse> {
  const response = await context.nativeMessenger.sendJson({
    request_id: context.idGenerator.generate(),
    message: { case: 'HealthCheck', value: {} },
  })
  if (!response?.success || response.response?.case !== 'HealthCheckResult')
    throw new Error(response?.error || 'Native Hostの接続確認に失敗しました')

  const stored = await context.browser.storage.get(['sync_summary'])
  const summary = stored.sync_summary
  return {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'statusResult',
      value: {
        status: {
          lastSync: typeof summary?.lastSync === 'string' ? summary.lastSync : '',
          totalSynced: typeof summary?.totalSynced === 'number' ? summary.totalSynced : 0,
          connectedExtensions: [context.extensionId],
          isRunning: true,
          connectionStatus: 'connected',
          errorMessage: '',
        },
      },
    },
  }
}
