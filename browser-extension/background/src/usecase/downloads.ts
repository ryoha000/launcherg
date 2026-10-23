import type { DownloadIntentEntry } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'
import { getDownloadIntent, incrementCompletedAndPushItem, logger, readAllDownloadIntents, removeDownloadIntent, stripDownloadItemFields, toDownloadIntentTs, updateDownloadIntent } from '@launcherg/shared'

const log = logger('background:downloads')
const RETRY_ALARM = 'retry_downloads'
const MAX_ATTEMPTS = 3

declare global {
  interface ServiceWorkerGlobalScope {
    __launchergDownloadsOnChangedHandler?: (delta: chrome.downloads.DownloadDelta) => Promise<void>
  }
}

function storeIdFromUrl(url: string): string | null {
  try {
    const parsed = new URL(url)
    return parsed.searchParams.get('productId') || parsed.searchParams.get('workno')
      || parsed.searchParams.get('pid') || parsed.pathname.split('/')[2] || null
  }
  catch { return null }
}

export async function startIntentDownloads(storeId: string, urls: string[]): Promise<number> {
  if (!urls.length)
    throw new Error('ダウンロードURLがありません。ストアのページを再読み込みしてください')
  const entry = await getDownloadIntent(storeId)
  if (!entry)
    throw new Error('ダウンロード情報が見つかりません')
  const ids = [...(entry.downloadIds ?? [])]
  const pending = [...urls]
  await updateDownloadIntent(storeId, { pendingUrls: pending })
  let started = 0
  for (const url of urls) {
    try {
      const id = await chrome.downloads.download({ url, saveAs: false })
      ids.push(id)
      pending.splice(pending.indexOf(url), 1)
      started++
      await updateDownloadIntent(storeId, { downloadIds: [...ids], pendingUrls: [...pending] })
    }
    catch (error) {
      await updateDownloadIntent(storeId, { lastError: String(error) })
    }
  }
  if (pending.length)
    throw new Error(`${pending.length}件のダウンロード開始に失敗しました。拡張機能から再試行できます`)
  return started
}

export function setupDownloadsHandler(context: HandlerContext): (manual?: boolean) => Promise<void> {
  const inFlight = new Set<string>()

  async function complete(storeId: string, entry: DownloadIntentEntry, manual = false): Promise<void> {
    if (inFlight.has(storeId) || entry.completed < entry.expected)
      return
    if (!manual && ((entry.attempts ?? 0) >= MAX_ATTEMPTS || (entry.nextRetryAt ?? 0) > Date.now()))
      return
    inFlight.add(storeId)
    const attempts = manual ? 1 : (entry.attempts ?? 0) + 1
    try {
      await updateDownloadIntent(storeId, { attempts, lastError: undefined })
      const intent = toDownloadIntentTs(entry)
      if (!intent)
        throw new Error('ダウンロード情報が正しくありません')
      const response = await context.nativeMessenger.sendJson({
        request_id: `download-${storeId}-${entry.startedAt}`,
        message: { case: 'DownloadsCompleted', value: {
          extension_id: context.extensionId,
          items: entry.items ?? [],
          intent,
        } },
      })
      if (!response?.success)
        throw new Error(response?.error || 'Launchergから処理結果を受信できませんでした')
      await removeDownloadIntent(storeId)
    }
    catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      await updateDownloadIntent(storeId, { attempts, lastError: message, nextRetryAt: Date.now() + attempts * 60_000 })
      log.error('インストール失敗。ダウンロード情報を保持します', { storeId, message })
      await context.browser.notifications.create({
        type: 'basic',
        iconUrl: context.browser.runtime.getURL('icons/icon32.png'),
        title: 'Launcherg',
        message: 'インストールに失敗しました。元ファイルを保持しています。拡張機能から再試行できます。',
      }).catch(error => log.error('通知に失敗しました', error))
    }
    finally { inFlight.delete(storeId) }
  }

  async function recordItem(item: chrome.downloads.DownloadItem): Promise<string | null> {
    const entries = await readAllDownloadIntents()
    const fromUrl = storeIdFromUrl(item.url)
    const storeId = Object.keys(entries).find(id => entries[id].downloadIds?.includes(item.id))
      ?? (fromUrl && entries[fromUrl] ? fromUrl : null)
    if (!storeId || Date.parse(item.startTime) < entries[storeId].startedAt - 1000)
      return null
    if (item.state === 'complete')
      await incrementCompletedAndPushItem(storeId, stripDownloadItemFields(item))
    else if (item.state === 'interrupted' && (!entries[storeId].downloadIds?.length || entries[storeId].downloadIds?.includes(item.id)))
      await updateDownloadIntent(storeId, { lastError: `ダウンロードが中断しました: ${item.error ?? '不明なエラー'}` })
    return storeId
  }

  const onChanged = async (delta: chrome.downloads.DownloadDelta) => {
    if (!delta.state || !['complete', 'interrupted'].includes(delta.state.current ?? ''))
      return
    try {
      const [item] = await chrome.downloads.search({ id: delta.id })
      if (!item)
        return
      const storeId = await recordItem(item)
      const entry = storeId ? await getDownloadIntent(storeId) : null
      if (storeId && entry)
        await complete(storeId, entry)
    }
    catch (error) { log.error('ダウンロード完了処理に失敗しました', error) }
  }

  async function runRetry(manual = false): Promise<void> {
    const entries = await readAllDownloadIntents()
    if (!Object.keys(entries).length)
      return
    // Reconcile completion events missed during a worker/browser restart.
    const startedAfter = new Date(Math.min(...Object.values(entries).map(entry => entry.startedAt)) - 1000).toISOString()
    const items = await chrome.downloads.search({ startedAfter })
    for (const item of items) {
      const storeId = await recordItem(item)
      if (manual && storeId && item.state === 'interrupted') {
        const entry = await getDownloadIntent(storeId)
        // A previous retry replaces this id; do not start the old failed item again.
        if (entry?.downloadIds?.length && !entry.downloadIds.includes(item.id))
          continue
        if (item.canResume) {
          await chrome.downloads.resume(item.id).catch(error => updateDownloadIntent(storeId, { lastError: String(error) }))
        }
        else {
          await updateDownloadIntent(storeId, {
            downloadIds: (entry?.downloadIds ?? []).filter(id => id !== item.id),
            pendingUrls: [...new Set([...(entry?.pendingUrls ?? []), item.url])],
          })
        }
      }
    }
    for (const [storeId, entry] of Object.entries(await readAllDownloadIntents())) {
      if (manual && entry.pendingUrls?.length)
        await startIntentDownloads(storeId, entry.pendingUrls).catch(error => log.error('再ダウンロードに失敗しました', error))
      await complete(storeId, entry, manual)
    }
  }

  let retryTail = Promise.resolve()
  function retry(manual = false): Promise<void> {
    const current = retryTail.catch(() => undefined).then(() => runRetry(manual))
    retryTail = current
    return current
  }

  chrome.downloads.onChanged.addListener(onChanged)
  ;(globalThis as ServiceWorkerGlobalScope).__launchergDownloadsOnChangedHandler = onChanged
  chrome.alarms.create(RETRY_ALARM, { periodInMinutes: 1 })
  chrome.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === RETRY_ALARM)
      void retry().catch(error => log.error('ダウンロード復旧に失敗しました', error))
  })
  void retry().catch(error => log.error('ダウンロード復旧に失敗しました', error))
  return retry
}
