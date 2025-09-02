import type { DownloadIntentTs, NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { incrementCompletedAndPushItem, logger, readAllDownloadIntents, removeDownloadIntent, stripDownloadItemFields, toDownloadIntentTs } from '@launcherg/shared'

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

      // storeId を URL から抽出（DLsite/DMM対応）
      const storeId = (() => {
        try {
          const u = new URL(item.url ?? '')
          // DLsite: https://play.dlsite.com/api/v3/download?workno=RJ01363269
          const workno = u.searchParams.get('workno')
          if (workno)
            return workno
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

      // 複数意図の管理マップから参照
      const intentsMap = await readAllDownloadIntents()
      const entry = storeId ? intentsMap[storeId] ?? null : null
      log.debug('storeId と intent を解決', { storeId, hasIntent: !!entry })

      if (storeId && entry) {
        // 進捗更新とファイル情報の集約（ヘルパー経由）
        const updated = await incrementCompletedAndPushItem(storeId, stripDownloadItemFields(item))
        const current = updated ?? entry

        log.debug('intent を更新', { storeId, completed: current.completed, expected: current.expected })

        if (current.completed >= current.expected) {
          // すべて完了: まとめて一度だけ送信
          const intentUnion: DownloadIntentTs | undefined = toDownloadIntentTs(current)

          const aggregateMsg: NativeMessageTs = {
            request_id: context.idGenerator.generate(),
            message: {
              case: 'DownloadsCompleted',
              value: {
                extension_id: context.extensionId,
                items: current.items ?? [],
                intent: intentUnion,
              },
            },
          }

          log.info('全パーツ完了 -> 集約メッセージを送信', { storeId, parts: (current.items ?? []).length })
          await context.nativeMessenger.sendJson(aggregateMsg)
          log.info('集約メッセージを送信しました', { storeId })

          // 後片付け
          await removeDownloadIntent(storeId)
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
    }
    catch (e) {
      log.error('downloads.onChanged handler error', e)
    }
  })
}
