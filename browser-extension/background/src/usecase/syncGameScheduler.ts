import type { DlsiteGame as ExtDlsiteGame, DmmGame as ExtDmmGame, EgsInfo as ExtEgsInfo } from '@launcherg/shared'
import type { NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'

export type SyncGameRequest
  = | { type: 'dmm', games: ExtDmmGame[] }
    | { type: 'dlsite', games: ExtDlsiteGame[] }

export function syncGame(context: HandlerContext, request: SyncGameRequest): Promise<void> {
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
    const res = await context.nativeMessenger.sendJson?.(payload)
    return res as NativeResponseTs | null
  }

  const notifyIfNew = async (count: number) => {
    if (!count || count < 1)
      return
    const iconUrl = context.browser.runtime.getURL('icons/icon32.png')
    await context.browser.notifications.create({
      type: 'basic',
      iconUrl,
      title: 'Launcherg',
      message: `新規に ${count} 件の作品を追加しました`,
    })
  }

  const processDmmBatch = async (games: ExtDmmGame[]) => {
    if (games.length === 0)
      return
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
      parent_pack: g.parentPack
        ? {
            store_id: g.parentPack.storeId,
            category: g.parentPack.category,
            subcategory: g.parentPack.subcategory,
          }
        : undefined,
    }))
    const msg = buildMessage({
      case: 'SyncDmmGames',
      value: {
        games: nativeGames,
        extension_id: context.extensionId,
      },
    })
    const res = await sendNative(msg)
    const count = res && res.success && res.response?.case === 'SyncGamesResult' ? (res.response.value?.new_count ?? 0) : 0
    await notifyIfNew(count)
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
    const res = await sendNative(msg)
    const count = res && res.success && res.response?.case === 'SyncGamesResult' ? (res.response.value?.new_count ?? 0) : 0
    await notifyIfNew(count)
  }

  return context.syncCoordinator.runExclusive(async () => {
    if (request.type === 'dmm') {
      await processDmmBatch(request.games)
      return
    }

    await processDlsiteBatch(request.games)
  })
}
