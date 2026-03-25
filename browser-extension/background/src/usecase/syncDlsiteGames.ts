import type { DlsiteSyncGamesRequest, ExtensionResponse } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'
import { syncGame } from './syncGameScheduler'

export async function handleSyncDlsiteGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DlsiteSyncGamesRequest,
): Promise<ExtensionResponse> {
  await syncGame(context, { type: 'dlsite', games: syncGamesRequest.games })

  return {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'syncGamesResult',
      value: { message: '同期を実行しました' },
    },
  }
}
