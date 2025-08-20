import type { DmmSyncGamesRequest } from '@launcherg/shared/proto/extension_internal'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import {
  ExtensionResponseSchema,
  SyncGamesResponseSchema,
} from '@launcherg/shared/proto/extension_internal'

export async function handleSyncDmmGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DmmSyncGamesRequest,
) {
  context.syncPool.add({ type: 'dmm', games: syncGamesRequest.games })

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
