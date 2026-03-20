import type { BrowserContext, Worker } from '@playwright/test'

/**
 * 拡張機能 ID を取得するヘルパー。
 * chrome://extensions には直接アクセスできないため、
 * 拡張機能のインストール後に開かれる Service Worker の URL から ID を抽出する。
 */
export async function getExtensionId(context: BrowserContext): Promise<string> {
  // Service Worker が登録されるまで待機
  let sw = context.serviceWorkers()[0]
  if (!sw) {
    sw = await context.waitForEvent('serviceworker')
  }
  // URL 形式: chrome-extension://<id>/background/background.js
  const url = sw.url()
  const match = url.match(/chrome-extension:\/\/([^/]+)\//)
  if (!match) {
    throw new Error(`Service Worker URL から拡張機能 ID を取得できませんでした: ${url}`)
  }
  return match[1]
}

/**
 * background Service Worker のコンテキストを取得する。
 * Playwright では ServiceWorker オブジェクトのみ取得可能で、
 * evaluate() で直接 spy を仕込むことができる。
 */
export async function getServiceWorker(context: BrowserContext): Promise<Worker> {
  let sw = context.serviceWorkers()[0]
  if (!sw) {
    sw = await context.waitForEvent('serviceworker')
  }
  return sw
}

/**
 * chrome.runtime.sendNativeMessage を Service Worker 内でモック関数に差し替え、
 * 呼び出された引数のコピーを記録する。
 * 呼び出し後は元の実装を呼ばずに即座にダミーレスポンスを返す。
 *
 * @returns spyをリセットして記録を取得するための関数群
 */
export async function setupSendNativeMessageSpy(sw: Worker): Promise<void> {
  await sw.evaluate(() => {
    // globalThis 上に spy 記録領域を初期化
    ;(globalThis as any).__nativeMessageCalls = []

    // chrome は Service Worker のグローバルスコープに存在する（@types/chrome 不要）
    const chromeRuntime = (globalThis as any).chrome.runtime
    chromeRuntime.sendNativeMessage = (
      target: string,
      message: unknown,
      callback?: (response: unknown) => void,
    ) => {
      ;(globalThis as any).__nativeMessageCalls.push({ target, message })
      // Native host がいない環境でも callback を呼ぶ（ダミー成功レスポンス）
      if (callback) {
        callback({ success: true, __mocked: true })
      }
      // 元の実装は呼ばない（Tauriホスト不要）
    }
  })
}

export async function setupDownloadsSpy(sw: Worker): Promise<void> {
  await sw.evaluate(() => {
    ;(globalThis as any).__downloadCalls = []
    ;(globalThis as any).__downloadItemsById = {}
    ;(globalThis as any).__nextDownloadId = 1

    const downloads = (globalThis as any).chrome.downloads
    downloads.download = (options: { url: string }, callback?: (downloadId?: number) => void) => {
      const id = (globalThis as any).__nextDownloadId++
      ;(globalThis as any).__downloadCalls.push({ id, url: options?.url, options })
      ;(globalThis as any).__downloadItemsById[id] = {
        id,
        url: options?.url,
        filename: `mock-${id}.bin`,
      }
      if (callback)
        callback(id)
      return Promise.resolve(id)
    }

    downloads.search = (query: { id?: number }, callback?: (items: any[]) => void) => {
      const item = query?.id ? (globalThis as any).__downloadItemsById[query.id] : undefined
      const result = item ? [item] : []
      if (callback)
        callback(result)
      return Promise.resolve(result)
    }
  })
}

/**
 * spy に記録された sendNativeMessage 呼び出し一覧を取得する。
 */
export async function getNativeMessageCalls(
  sw: Worker,
): Promise<Array<{ target: string, message: unknown }>> {
  return sw.evaluate(() => {
    return (globalThis as any).__nativeMessageCalls ?? []
  })
}

/**
 * spy の呼び出し記録をリセットする。
 */
export async function resetNativeMessageSpy(sw: Worker): Promise<void> {
  await sw.evaluate(() => {
    ;(globalThis as any).__nativeMessageCalls = []
  })
}

/**
 * spy に新しい呼び出しが届くまで最大 timeoutMs 待機する。
 * ポーリング間隔は 500ms。
 */
export async function waitForNativeMessageCall(
  sw: Worker,
  predicate: (calls: Array<{ target: string, message: unknown }>) => boolean,
  timeoutMs = 30_000,
): Promise<Array<{ target: string, message: unknown }>> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const calls = await getNativeMessageCalls(sw)
    if (predicate(calls)) {
      return calls
    }
    await new Promise(r => setTimeout(r, 500))
  }
  const calls = await getNativeMessageCalls(sw)
  throw new Error(
    `waitForNativeMessageCall: タイムアウト (${timeoutMs}ms) 。記録された呼び出し: ${JSON.stringify(calls)}`,
  )
}

export async function getDownloadIntents(sw: Worker): Promise<Record<string, unknown>> {
  return sw.evaluate(() => {
    return new Promise<Record<string, unknown>>((resolve) => {
      chrome.storage.local.get(['download_intents'], (result) => {
        resolve((result?.download_intents as Record<string, unknown>) ?? {})
      })
    })
  })
}

export async function clearDownloadIntents(sw: Worker): Promise<void> {
  await sw.evaluate(() => {
    return new Promise<void>((resolve) => {
      chrome.storage.local.set({ download_intents: {} }, () => resolve())
    })
  })
}

export async function getDownloadCalls(sw: Worker): Promise<Array<{ id: number, url: string }>> {
  return sw.evaluate(() => {
    return (globalThis as any).__downloadCalls ?? []
  })
}

export async function resetDownloadSpy(sw: Worker): Promise<void> {
  await sw.evaluate(() => {
    ;(globalThis as any).__downloadCalls = []
    ;(globalThis as any).__downloadItemsById = {}
    ;(globalThis as any).__nextDownloadId = 1
  })
}

export async function waitForDownloadCalls(
  sw: Worker,
  predicate: (calls: Array<{ id: number, url: string }>) => boolean,
  timeoutMs = 30_000,
): Promise<Array<{ id: number, url: string }>> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const calls = await getDownloadCalls(sw)
    if (predicate(calls))
      return calls
    await new Promise(r => setTimeout(r, 500))
  }

  const calls = await getDownloadCalls(sw)
  throw new Error(
    `waitForDownloadCalls: タイムアウト (${timeoutMs}ms) 。記録された download 呼び出し: ${JSON.stringify(calls)}`,
  )
}

export async function emitDownloadComplete(
  sw: Worker,
  downloadId: number,
  item: { url: string, filename: string },
): Promise<void> {
  await sw.evaluate(async ({ downloadId, item }) => {
    ;(globalThis as any).__downloadItemsById = {
      ...((globalThis as any).__downloadItemsById ?? {}),
      [downloadId]: {
        id: downloadId,
        url: item.url,
        filename: item.filename,
      },
    }

    const handler = (globalThis as any).__launchergDownloadsOnChangedHandler
    if (typeof handler !== 'function')
      throw new Error('downloads.onChanged handler が見つかりません')

    await handler({
      id: downloadId,
      state: { current: 'complete', previous: 'in_progress' },
    })
  }, { downloadId, item })
}

export async function waitForDownloadIntent(
  sw: Worker,
  storeId: string,
  timeoutMs = 30_000,
): Promise<unknown> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const intents = await getDownloadIntents(sw)
    const entry = intents[storeId]
    if (entry) {
      return entry
    }
    await new Promise(r => setTimeout(r, 500))
  }

  const intents = await getDownloadIntents(sw)
  throw new Error(
    `waitForDownloadIntent: タイムアウト (${timeoutMs}ms) 。記録された intent: ${JSON.stringify(intents)}`,
  )
}
