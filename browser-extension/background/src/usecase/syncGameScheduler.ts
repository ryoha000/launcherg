import type { DlsiteGame as ExtDlsiteGame, DmmGame as ExtDmmGame, EgsInfo as ExtEgsInfo } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'

export type SyncGameRequest
  = | { type: 'dmm', games: ExtDmmGame[] }
    | { type: 'dlsite', games: ExtDlsiteGame[] }

export function syncGame(context: HandlerContext, request: SyncGameRequest): Promise<void> {
  const record = (stage: string, count?: number, detail = '') => {
    void context.diagnostics?.record({ timestamp: new Date().toISOString(), store: request.type, stage, trigger: 'background', page_kind: 'background', count, detail }).catch(() => {})
  }
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
    record('native_requested', request.games.length)
    const res = await context.nativeMessenger.sendJson(payload)
    if (!res?.success)
      throw new Error(res?.error || 'Launchergから同期結果を受信できませんでした')
    if (res.response?.case !== 'SyncGamesResult')
      throw new Error('Launchergから予期しない同期結果を受信しました')
    if (res.response.value.error_count > 0)
      throw new Error(res.response.value.errors.join('\n') || `${res.response.value.error_count}件の同期に失敗しました`)
    record('native_succeeded', res.response.value.success_count)
    // This is presentation metadata. Storage/notification failures must not turn a committed sync into a failure.
    try {
      const stored = await context.browser.storage.get(['sync_summary'])
      await context.browser.storage.set({ sync_summary: {
        lastSync: new Date().toISOString(),
        totalSynced: (Number(stored.sync_summary?.totalSynced) || 0) + res.response.value.new_count,
      } })
    }
    catch (error) { record('status_save_failed', undefined, String(error)) }
    return res
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
    }).catch(error => record('notification_failed', undefined, String(error)))
  }

  const processDmmBatch = async (games: ExtDmmGame[]) => {
    if (games.length === 0)
      return
    record('metadata_requested', games.length)
    const resolved = await context.egsResolver.resolveForDmmBulk(
      games.map(g => ({ storeId: g.id, category: g.category, subcategory: g.subcategory })),
    )
    record('metadata_received', resolved.filter(Boolean).length)
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
    record('metadata_requested', games.length)
    const resolved = await context.egsResolver.resolveForDlsiteBulk(
      games.map(g => ({ storeId: g.id, category: g.category })),
    )
    record('metadata_received', resolved.filter(Boolean).length)
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

  record('sync_queued', request.games.length)
  return context.syncCoordinator.runExclusive(async () => {
    if (request.type === 'dmm') {
      await processDmmBatch(request.games)
      return
    }

    await processDlsiteBatch(request.games)
  }).catch((error: unknown) => {
    record('background_sync_failed', undefined, error instanceof Error ? error.message : String(error))
    throw error
  })
}
