import type { DlsiteSyncGamesRequest } from '@launcherg/shared/proto/extension_internal'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { ExtensionResponseSchema, SyncGamesResponseSchema } from '@launcherg/shared/proto/extension_internal'

export async function handleSyncDlsiteGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DlsiteSyncGamesRequest,
) {
  context.syncPool.add({ type: 'dlsite', games: syncGamesRequest.games })

  return create(ExtensionResponseSchema, {
    requestId,
    success: true,
    error: '',
    response: {
      case: 'syncGamesResult',
      value: create(SyncGamesResponseSchema, {
        message: `プールに追加しました`,
      }),
    },
  })
}
