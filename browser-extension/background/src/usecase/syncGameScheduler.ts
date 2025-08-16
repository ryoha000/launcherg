import type { DlsiteGame as ExtDlsiteGame, DmmGame as ExtDmmGame, EgsInfo as ExtEgsInfo } from '@launcherg/shared/proto/extension_internal'
import type { DlsiteGame as NativeDlsiteGame, DmmGame as NativeDmmGame, EgsInfo as NativeEgsInfo, NativeMessage } from '@launcherg/shared/proto/native_messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import {
  DlsiteGameSchema as NativeDlsiteGameSchema,
  DlsiteSyncGamesRequestSchema as NativeDlsiteSyncGamesRequestSchema,
  DmmGameSchema as NativeDmmGameSchema,
  DmmSyncGamesRequestSchema as NativeDmmSyncGamesRequestSchema,
  EgsInfoSchema as NativeEgsInfoSchema,
  NativeMessageSchema,
} from '@launcherg/shared/proto/native_messaging'

const SYNC_GAME_ALARM = 'sync_game'

export { SYNC_GAME_ALARM }

export function syncGame(context: HandlerContext): Promise<void> {
  const toNativeEgsInfo = (egs: ExtEgsInfo | null): NativeEgsInfo | undefined => {
    if (!egs)
      return undefined
    return create(NativeEgsInfoSchema, {
      erogamescapeId: egs.erogamescapeId,
      gamename: egs.gamename,
      gamenameRuby: egs.gamenameRuby,
      brandname: egs.brandname,
      brandnameRuby: egs.brandnameRuby,
      sellday: egs.sellday,
      isNukige: egs.isNukige,
    })
  }

  const buildNativeMessage = (message: NativeMessage['message']): NativeMessage => create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, { seconds: BigInt(Math.floor(Date.now() / 1000)) }),
    requestId: context.idGenerator.generate(),
    message,
  })

  const sendNative = async (nativeMessage: NativeMessage) => {
    await context.nativeMessenger.send(nativeMessage)
  }

  const processDmmBatch = async (games: ExtDmmGame[]) => {
    if (games.length === 0)
      return
    const resolved = await context.egsResolver.resolveForDmmBulk(
      games.map(g => ({ storeId: g.id, category: g.category, subcategory: g.subcategory })),
    )
    const nativeGames: NativeDmmGame[] = games.map((g, i) => create(NativeDmmGameSchema, {
      id: g.id,
      category: g.category,
      subcategory: g.subcategory,
      egsInfo: toNativeEgsInfo(resolved[i]),
    }))
    const msg = buildNativeMessage({
      case: 'syncDmmGames',
      value: create(NativeDmmSyncGamesRequestSchema, {
        games: nativeGames,
        extensionId: context.extensionId,
      }),
    })
    await sendNative(msg)
  }

  const processDlsiteBatch = async (games: ExtDlsiteGame[]) => {
    if (games.length === 0)
      return
    const resolved = await context.egsResolver.resolveForDlsiteBulk(
      games.map(g => ({ storeId: g.id, category: g.category })),
    )
    const nativeGames: NativeDlsiteGame[] = games.map((g, i) => create(NativeDlsiteGameSchema, {
      id: g.id,
      category: g.category,
      egsInfo: toNativeEgsInfo(resolved[i]),
    }))
    const msg = buildNativeMessage({
      case: 'syncDlsiteGames',
      value: create(NativeDlsiteSyncGamesRequestSchema, {
        games: nativeGames,
        extensionId: context.extensionId,
      }),
    })
    await sendNative(msg)
  }

  return context.syncPool.sync(async (items) => {
    const dmmGames = items
      .filter((it): it is { type: 'dmm', games: ExtDmmGame[] } => it.type === 'dmm')
      .flatMap(it => it.games)
    const dlsiteGames = items
      .filter((it): it is { type: 'dlsite', games: ExtDlsiteGame[] } => it.type === 'dlsite')
      .flatMap(it => it.games)
    await processDmmBatch(dmmGames)
    await processDlsiteBatch(dlsiteGames)
  })
}
