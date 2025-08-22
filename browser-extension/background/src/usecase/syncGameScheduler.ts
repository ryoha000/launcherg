import type { DlsiteGame as ExtDlsiteGame, DmmGame as ExtDmmGame, EgsInfo as ExtEgsInfo } from '@launcherg/shared/proto/extension_internal'
import type { HandlerContext } from '../shared/types'

const SYNC_GAME_ALARM = 'sync_game'

export { SYNC_GAME_ALARM }

export function syncGame(context: HandlerContext): Promise<void> {
  const toTypeshareEgsInfo = (egs: ExtEgsInfo | null) => {
    if (!egs)
      return undefined
    return {
      erogamescape_id: egs.erogamescapeId,
      gamename: egs.gamename,
      gamename_ruby: egs.gamenameRuby,
      brandname: egs.brandname,
      brandname_ruby: egs.brandnameRuby,
      sellday: egs.sellday,
      is_nukige: egs.isNukige,
    }
  }

  const buildMessage = (message: any) => ({
    request_id: context.idGenerator.generate(),
    message,
  })

  const sendNative = async (payload: { request_id: string, message: any }) => {
    await context.nativeMessenger.sendJson?.(payload)
  }

  const processDmmBatch = async (games: ExtDmmGame[]) => {
    if (games.length === 0)
      return
    // EGS resolve (保持)
    const resolved = await context.egsResolver.resolveForDmmBulk(
      games.map(g => ({ storeId: g.id, category: g.category, subcategory: g.subcategory })),
    )
    const nativeGames = games.map((g, i) => ({
      id: g.id,
      category: g.category,
      subcategory: g.subcategory,
      egs_info: toTypeshareEgsInfo(resolved[i]) ?? undefined,
      title: g.title,
      image_url: g.imageUrl,
    }))
    const msg = buildMessage({
      case: 'SyncDmmGames',
      value: {
        games: nativeGames,
        extension_id: context.extensionId,
      },
    })
    await sendNative(msg)
  }

  const processDlsiteBatch = async (games: ExtDlsiteGame[]) => {
    if (games.length === 0)
      return
    const resolved = await context.egsResolver.resolveForDlsiteBulk(
      games.map(g => ({ storeId: g.id, category: g.category })),
    )
    const nativeGames = games.map((g, i) => ({
      id: g.id,
      category: g.category,
      egs_info: toTypeshareEgsInfo(resolved[i]) ?? undefined,
      title: g.title,
      image_url: g.imageUrl,
    }))
    const msg = buildMessage({
      case: 'SyncDlsiteGames',
      value: {
        games: nativeGames,
        extension_id: context.extensionId,
      },
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
