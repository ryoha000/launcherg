import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { logger } from '@launcherg/shared'

const log = logger('background:downloads')

export function setupDownloadsHandler(context: HandlerContext): void {
  if (
    typeof chrome === 'undefined'
    || !chrome.downloads
    || !chrome.downloads.onChanged
    || typeof chrome.downloads.onChanged.addListener !== 'function'
  ) {
    log.debug('この環境では downloads API が利用できないため、onChanged の登録をスキップします')
    return
  }

  log.debug('downloads.onChanged リスナーを登録します')

  chrome.downloads.onChanged.addListener(async (delta) => {
    try {
      log.debug('downloads.onChanged を受信', delta)

      if (!delta.state || delta.state.current !== 'complete') {
        log.debug('完了以外の状態のためスキップ', { id: delta.id, state: delta.state?.current })
        return
      }

      const item = await new Promise<chrome.downloads.DownloadItem | null>((resolve) => {
        chrome.downloads.search({ id: delta.id }, items => resolve(items?.[0] ?? null))
      })
      if (!item || !item.filename) {
        log.debug('ダウンロード項目が取得できない、またはファイル名が空のためスキップ', { id: delta.id })
        return
      }

      log.debug('ダウンロード項目を解決しました', { id: item.id, filename: item.filename, size: item.fileSize })

      // storeId を URL から抽出
      const storeId = (() => {
        try {
          const u = new URL(item.url ?? '')
          const pid = u.searchParams.get('pid')
          if (pid)
            return pid
          const parts = (u.pathname || '').split('/')
          return parts.length >= 3 ? parts[2] : null
        }
        catch {
          const url = item.url ?? ''
          const m = url.match(/\/(\w+)\//)
          return m ? m[1] : null
        }
      })()

      // 複数意図の管理マップを読み込み
      const intentsMap = await new Promise<any>((resolve) => {
        chrome.storage.local.get(['download_intents'], res => resolve(res?.download_intents ?? {}))
      })
      const entry = storeId ? intentsMap[storeId] ?? null : null
      log.debug('storeId と intent を解決', { storeId, hasIntent: !!entry })

      if (storeId && entry) {
        // 進捗更新とファイル情報の集約
        entry.completed = (entry.completed ?? 0) + 1
        entry.items = Array.isArray(entry.items) ? entry.items : []
        entry.items.push({
          id: item.id,
          filename: item.filename,
          mime: item.mime,
          file_size: item.fileSize,
          url: item.url,
          start_time: item.startTime,
          end_time: item.endTime,
        })
        intentsMap[storeId] = entry
        await new Promise<void>(resolve => chrome.storage.local.set({ download_intents: intentsMap }, () => resolve()))

        log.debug('intent を更新', { storeId, completed: entry.completed, expected: entry.expected })

        if (entry.completed >= entry.expected) {
          // すべて完了: まとめて一度だけ送信
          const aggregateMsg: NativeMessageTs = {
            request_id: context.idGenerator.generate(),
            message: {
              case: 'DownloadsCompleted',
              value: {
                extension_id: context.extensionId,
                items: entry.items,
                intent: {
                  store: String(entry.store ?? ''),
                  game_store_id: String(entry.game?.storeId ?? ''),
                  game_category: String(entry.game?.category ?? ''),
                  game_subcategory: String(entry.game?.subcategory ?? ''),
                  parent_pack_store_id: entry.parentPack?.storeId ?? undefined,
                  parent_pack_category: entry.parentPack?.category ?? undefined,
                  parent_pack_subcategory: entry.parentPack?.subcategory ?? undefined,
                },
              },
            },
          }

          log.info('全パーツ完了 -> 集約メッセージを送信', { storeId, parts: entry.items.length })
          await context.nativeMessenger.sendJson(aggregateMsg)
          log.info('集約メッセージを送信しました', { storeId })

          // 後片付け
          delete intentsMap[storeId]
          await new Promise<void>(resolve => chrome.storage.local.set({ download_intents: intentsMap }, () => resolve()))
        }
      }
      else {
        // フォールバック: intent が無い/storeId 不明時は単発で送る
        const fallbackMsg: NativeMessageTs = {
          request_id: context.idGenerator.generate(),
          message: {
            case: 'DownloadsCompleted',
            value: {
              extension_id: context.extensionId,
              items: [{
                id: item.id,
                filename: item.filename,
                mime: item.mime,
                url: item.url,
                start_time: item.startTime,
                end_time: item.endTime,
              }],
              intent: undefined,
            },
          },
        }
        log.info('intent なしのため単発送信', { storeId })
        await context.nativeMessenger.sendJson(fallbackMsg)
      }

      // 旧キー互換のクリーンアップは不要
    }
    catch (e) {
      log.error('downloads.onChanged handler error', e)
    }
  })
}
