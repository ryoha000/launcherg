import type { DmmSyncGamesRequest, ExtensionResponse } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'

export async function handleSyncDmmGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DmmSyncGamesRequest,
): Promise<ExtensionResponse> {
  context.syncPool.add({ type: 'dmm', games: syncGamesRequest.games })

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
