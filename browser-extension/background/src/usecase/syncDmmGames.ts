import type { DmmSyncGamesRequest, ExtensionResponse } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'
import { syncGame } from './syncGameScheduler'

export async function handleSyncDmmGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DmmSyncGamesRequest,
): Promise<ExtensionResponse> {
  await syncGame(context, { type: 'dmm', games: syncGamesRequest.games })

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
