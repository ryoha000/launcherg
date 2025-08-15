import type { DlsiteSyncGamesRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessage } from '@launcherg/shared/proto/native_messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import {
  ExtensionResponseSchema,
  SyncGamesResponseSchema,
  SyncResultSchema,
} from '@launcherg/shared/proto/extension_internal'
import {
  DlsiteSyncGamesRequestSchema as NativeDlsiteSyncGamesRequestSchema,
  NativeMessageSchema,
} from '@launcherg/shared/proto/native_messaging'

export async function handleSyncDlsiteGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DlsiteSyncGamesRequest,
) {
  const resolvedGames = await Promise.all(
    syncGamesRequest.games.map(async (g) => {
      const egs = await context.egsResolver.resolveForDlsite(g.id, g.category)
      return egs
        ? { id: g.id, category: g.category, egsInfo: {
            erogamescapeId: egs.erogamescapeId,
            gamename: egs.gamename,
            gamenameRuby: egs.gamenameRuby,
            brandname: egs.brandname,
            brandnameRuby: egs.brandnameRuby,
            sellday: egs.sellday,
            isNukige: egs.isNukige,
          } }
        : { id: g.id, category: g.category }
    }),
  )

  const nativeSyncRequest = create(NativeDlsiteSyncGamesRequestSchema, {
    games: resolvedGames,
    extensionId: context.extensionId,
  })

  const nativeMessage = create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, {
      seconds: BigInt(Math.floor(Date.now() / 1000)),
    }),
    requestId: context.idGenerator.generate(),
    message: {
      case: 'syncDlsiteGames',
      value: nativeSyncRequest,
    },
  }) as NativeMessage

  const nativeResponse = await context.nativeMessenger.send(nativeMessage)

  if (nativeResponse && nativeResponse.success) {
    let syncResult
    if (nativeResponse.response.case === 'syncGamesResult') {
      const syncBatchResult = nativeResponse.response.value
      syncResult = create(SyncResultSchema, {
        successCount: Number(syncBatchResult.successCount),
        errorCount: Number(syncBatchResult.errorCount),
        errors: syncBatchResult.errors,
        syncedGames: syncBatchResult.syncedGames,
      })
      const success = Number(syncBatchResult.successCount || 0)
      await context.aggregation.record(success)
    }

    return create(ExtensionResponseSchema, {
      requestId,
      success: true,
      error: '',
      response: {
        case: 'syncGamesResult',
        value: create(SyncGamesResponseSchema, {
          result: syncResult,
          message: `DLsiteから${syncGamesRequest.games.length}個のゲームを同期しました`,
        }),
      },
    })
  }

  return create(ExtensionResponseSchema, {
    requestId,
    success: false,
    error: nativeResponse?.error || 'Native host returned error',
    response: {
      case: 'syncGamesResult',
      value: create(SyncGamesResponseSchema, {
        message: `DLsiteの同期に失敗しました`,
      }),
    },
  })
}
