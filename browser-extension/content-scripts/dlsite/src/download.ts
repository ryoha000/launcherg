import { logger, reportIntegrationDiagnostic, setDownloadIntent, showInPageNotification } from '@launcherg/shared'

const log = logger('dlsite-download')

export interface LaunchergDownloadParamDlsite {
  type: 'download'
  value: {
    game: { storeId: string }
  }
}

export function parseLaunchergParam(href: string = window.location.href): LaunchergDownloadParamDlsite | null {
  try {
    const url = new URL(href)
    const raw = url.searchParams.get('launcherg')
    if (!raw)
      return null
    const decoded = decodeURIComponent(raw)
    const parsed = JSON.parse(decoded) as unknown
    if (
      typeof parsed === 'object' && parsed !== null
      && (parsed as any).type === 'download'
      && typeof (parsed as any).value === 'object' && (parsed as any).value !== null
      && typeof (parsed as any).value.game === 'object' && (parsed as any).value.game !== null
      && typeof (parsed as any).value.game.storeId === 'string'
    ) {
      return parsed as LaunchergDownloadParamDlsite
    }
    return null
  }
  catch (e) {
    log.debug('Failed to parse launcherg param', e)
    return null
  }
}

export function closeCurrentTab(): void {
  try {
    chrome.runtime.sendMessage({ type: 'close_current_tab' })
  }
  catch {}
}

function isDetailPage(storeId: string): boolean {
  try {
    const u = new URL(window.location.href)
    // Current pages normalize /work/ID/tree to /work/ID.
    return /^\/work\/([^/]+)(?:\/|$)/.exec(u.pathname)?.[1] === storeId
  }
  catch {
    return false
  }
}

export function findZipDownloadUrl(storeId: string, root: Document | HTMLElement = document): string | null {
  for (const link of root.querySelectorAll<HTMLAnchorElement>('a[href]')) {
    if (!(link.textContent || '').replace(/\s+/g, '').includes('ZIPダウンロード'))
      continue
    try {
      const url = new URL(link.href, window.location.href)
      if (url.origin === window.location.origin && /^\/api\/v\d+\/download$/.test(url.pathname)
        && url.searchParams.get('workno') === storeId) {
        return url.toString()
      }
    }
    catch {}
  }
  return null
}

async function waitForZipDownloadUrl(storeId: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    const observer = new MutationObserver(() => check())
    const timeout = setTimeout(() => {
      observer.disconnect()
      reject(new Error('DLsite: ZIPダウンロードリンクが見つかりませんでした。ログイン状態を確認してページを再読み込みしてください'))
    }, 30_000)
    const cleanup = () => {
      observer.disconnect()
      clearTimeout(timeout)
    }
    function check(): void {
      if (!isDetailPage(storeId)) {
        cleanup()
        reject(new Error('DLsite: 作品ページが切り替わったためダウンロードを中止しました'))
        return
      }
      const url = findZipDownloadUrl(storeId)
      if (url) {
        cleanup()
        resolve(url)
      }
    }
    observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['href'] })
    check()
  })
}

async function saveDownloadIntent(storeId: string): Promise<void> {
  await setDownloadIntent(storeId, {
    store: 'DLsite',
    game: { storeId },
    expected: 1,
    completed: 0,
    startedAt: Date.now(),
  })
}

export async function startZipDownloadOnDetail(storeId: string): Promise<void> {
  const url = await waitForZipDownloadUrl(storeId)
  reportIntegrationDiagnostic('dlsite', { stage: 'download_link_ready' })
  await saveDownloadIntent(storeId)
  const response = await chrome.runtime.sendMessage({ type: 'start_dlsite_downloads', payload: { storeId, urls: [url] } })
  if (!response?.success || response.startedCount !== 1)
    throw new Error(response?.error || 'DLsite: ダウンロード開始に失敗しました')
  reportIntegrationDiagnostic('dlsite', { stage: 'download_started', count: 1 })
  showInPageNotification('DLsite: ダウンロードを開始しました', 'success')
}

export async function initLaunchergDownloadOnceForUrl(url: string, mark: (url: string) => void, isMarked: (url: string) => boolean): Promise<void> {
  const p = parseLaunchergParam(url)
  if (!p || p.type !== 'download')
    return

  const storeId = p.value.game.storeId
  if (!storeId)
    return

  // 詳細ページ以外では何もしない（リダイレクト/無限スクロールを行わない）
  if (!isDetailPage(storeId))
    return

  // The site's URL normalization must not start the same work twice.
  const key = `dlsite-download:${storeId}`
  if (isMarked(key))
    return
  mark(key)
  log.info('Launcherg download param detected - DLsite flow start')
  reportIntegrationDiagnostic('dlsite', { stage: 'download_requested' })

  try {
    await startZipDownloadOnDetail(storeId)
    closeCurrentTab()
  }
  catch (error) {
    reportIntegrationDiagnostic('dlsite', { stage: 'download_failed', detail: error instanceof Error ? error.message : String(error) })
    showInPageNotification(error instanceof Error ? error.message : String(error), 'error')
  }
}
