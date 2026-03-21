import { logger, setDownloadIntent, showInPageNotification, waitForPageLoad } from '@launcherg/shared'

const log = logger('dlsite-download')

export interface LaunchergDownloadParamDlsite {
  type: 'download'
  value: {
    game: { storeId: string }
  }
}

export function parseLaunchergParam(): LaunchergDownloadParamDlsite | null {
  try {
    const url = new URL(window.location.href)
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

function isDetailPage(): boolean {
  try {
    const u = new URL(window.location.href)
    // 例: https://play.dlsite.com/work/RJ01234567/tree
    return /\/work\/[^/]+\//.test(u.pathname)
  }
  catch {
    return false
  }
}

export function findZipDownloadButton(root: Document | HTMLElement = document): HTMLElement | null {
  const scope = root instanceof Document ? (root.getElementById('root') ?? root) : root
  const candidates = Array.from(scope.querySelectorAll('a,button,[role="button"]')) as HTMLElement[]

  for (const el of candidates) {
    const text = (el.textContent || '').replace(/\s+/g, ' ').trim()
    if (text.includes('ZIPダウンロード'))
      return el
  }
  return null
}

async function saveDownloadIntent(storeId: string): Promise<void> {
  try {
    await setDownloadIntent(storeId, {
      store: 'DLsite',
      game: { storeId },
      expected: 1,
      completed: 0,
      startedAt: Date.now(),
    })
  }
  catch (e) {
    log.debug('Failed to save download intent', e)
  }
}

export async function clickZipDownloadOnDetail(storeId: string): Promise<void> {
  await waitForPageLoad(1000)
  const button = findZipDownloadButton(document)
  if (!button) {
    showInPageNotification('DLsite: ZIPボタンが見つかりませんでした', 'error')
    return
  }
  try {
    button.scrollIntoView({ block: 'center' })
  }
  catch {}
  await saveDownloadIntent(storeId)
  await new Promise(r => setTimeout(r, 200))
  ;(button as HTMLElement).click()
  showInPageNotification('DLsite: ダウンロードを開始しました', 'success')
}

export async function initLaunchergDownloadOnceForUrl(url: string, mark: (url: string) => void, isMarked: (url: string) => boolean): Promise<void> {
  const p = parseLaunchergParam()
  if (!p || p.type !== 'download')
    return

  const storeId = p.value.game.storeId
  if (!storeId)
    return

  // 詳細ページ以外では何もしない（リダイレクト/無限スクロールを行わない）
  if (!isDetailPage())
    return

  if (isMarked(url))
    return
  mark(url)
  log.info('Launcherg download param detected - DLsite flow start')

  try {
    await clickZipDownloadOnDetail(storeId)
  }
  finally {
    closeCurrentTab()
  }
}
