import type { DlsiteSyncGamesRequest, ExtensionResponse } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'

export async function handleSyncDlsiteGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DlsiteSyncGamesRequest,
): Promise<ExtensionResponse> {
  context.syncPool.add({ type: 'dlsite', games: syncGamesRequest.games })

  return {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'syncGamesResult',
      value: { message: 'プールに追加しました' },
    },
  }
}
