import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { incrementCompletedAndPushItem, logger, readAllDownloadIntents, removeDownloadIntent, stripDownloadItemFields, toDownloadIntentTs } from '@launcherg/shared'

const log = logger('background:downloads')

declare global {
  interface Window {
    __launchergDownloadsOnChangedHandler?: (delta: chrome.downloads.DownloadDelta) => void | Promise<void>
  }
  /* eslint-disable-next-line ts/consistent-type-definitions */
  interface ServiceWorkerGlobalScope {
    __launchergDownloadsOnChangedHandler?: (delta: chrome.downloads.DownloadDelta) => void | Promise<void>
  }
}

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

  const onChanged = async (delta: chrome.downloads.DownloadDelta) => {
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
          const productId = u.searchParams.get('productId')
          if (productId)
            return productId
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
          const intentUnion = toDownloadIntentTs(current)
          if (!intentUnion) {
            log.error('intent が取得できない、または未定義のためスキップ', { storeId })
            return
          }

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
        log.error('intent が取得できない、または未定義のためスキップ', { storeId })
      }
    }
    catch (e) {
      log.error('downloads.onChanged handler error', e)
    }
  }

  chrome.downloads.onChanged.addListener(onChanged)
  ;(globalThis as ServiceWorkerGlobalScope).__launchergDownloadsOnChangedHandler = onChanged
}
