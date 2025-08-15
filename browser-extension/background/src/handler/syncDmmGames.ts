import type { DmmSyncGamesRequest } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessage } from '@launcherg/shared/proto/native_messaging'
import type { HandlerContext } from './handler'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import {

  ExtensionResponseSchema,
  SyncGamesResponseSchema,
  SyncResultSchema,
} from '@launcherg/shared/proto/extension_internal'
import {
  DmmSyncGamesRequestSchema as NativeDmmSyncGamesRequestSchema,

  NativeMessageSchema,
} from '@launcherg/shared/proto/native_messaging'

export async function handleSyncDmmGames(
  context: HandlerContext,
  requestId: string,
  syncGamesRequest: DmmSyncGamesRequest,
) {
  const resolvedGames = await Promise.all(
    syncGamesRequest.games.map(async (g) => {
      const egs = await context.resolveEgsForDmm(g.id, g.category, g.subcategory)
      return egs
        ? { id: g.id, category: g.category, subcategory: g.subcategory, egsInfo: {
            erogamescapeId: egs.erogamescapeId,
            gamename: egs.gamename,
            gamenameRuby: egs.gamenameRuby,
            brandname: egs.brandname,
            brandnameRuby: egs.brandnameRuby,
            sellday: egs.sellday,
            isNukige: egs.isNukige,
          } }
        : { id: g.id, category: g.category, subcategory: g.subcategory }
    }),
  )

  const nativeSyncRequest = create(NativeDmmSyncGamesRequestSchema, {
    games: resolvedGames,
    extensionId: context.extensionId,
  })

  const nativeMessage = create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, {
      seconds: BigInt(Math.floor(Date.now() / 1000)),
    }),
    requestId: context.generateRequestId(),
    message: {
      case: 'syncDmmGames',
      value: nativeSyncRequest,
    },
  }) as NativeMessage

  const nativeResponse = await context.sendNativeProtobufMessage(
    context.nativeHostName,
    nativeMessage,
  )

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
      await context.recordSyncAggregation(success)
    }

    return create(ExtensionResponseSchema, {
      requestId,
      success: true,
      error: '',
      response: {
        case: 'syncGamesResult',
        value: create(SyncGamesResponseSchema, {
          result: syncResult,
          message: `DMMから${syncGamesRequest.games.length}個のゲームを同期しました`,
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
        message: `DMMの同期に失敗しました`,
      }),
    },
  })
}
