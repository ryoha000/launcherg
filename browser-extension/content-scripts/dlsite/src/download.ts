import { logger, showInPageNotification, waitForPageLoad } from '@launcherg/shared'
import { extractAllGames, extractGameContainers, extractGameDataFromContainer } from './dom-extractor'

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
    const key = 'download_intents'
    const current = await new Promise<any>(resolve => chrome.storage.local.get([key], res => resolve(res?.[key] ?? {})))
    current[storeId] = {
      store: 'DLsite',
      game: { storeId },
      expected: 1,
      completed: 0,
      startedAt: Date.now(),
    }
    await chrome.storage.local.set({ [key]: current })
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

export async function performNavigateToDetailByStoreId(storeId: string, options?: { maxTries?: number, perTryWaitMs?: number }): Promise<boolean> {
  const maxTries = options?.maxTries ?? 30
  const perTryWaitMs = options?.perTryWaitMs ?? 400

  // まず現在描画済みから探す
  const initial = extractAllGames()
  const foundInitial = initial.find(g => g.storeId === storeId)
  if (foundInitial) {
    const containers = extractGameContainers()
    for (const el of Array.from(containers)) {
      const data = extractGameDataFromContainer(el, 0)
      if (data && data.storeId === storeId) {
        try {
          (el as HTMLElement).scrollIntoView({ block: 'center' })
        }
        catch {}
        await new Promise(r => setTimeout(r, 150))
        ;(el as HTMLElement).click()
        return true
      }
    }
  }

  // 無限スクロールして探索
  for (let i = 0; i < maxTries; i++) {
    try {
      window.scrollBy({ top: window.innerHeight * 0.9, behavior: 'auto' })
    }
    catch {}
    await new Promise(r => setTimeout(r, perTryWaitMs))

    const containers = extractGameContainers()
    for (const el of Array.from(containers)) {
      const data = extractGameDataFromContainer(el, 0)
      if (data && data.storeId === storeId) {
        try {
          (el as HTMLElement).scrollIntoView({ block: 'center' })
        }
        catch {}
        await new Promise(r => setTimeout(r, 50))
        ;(el as HTMLElement).click()
        return true
      }
    }
  }

  showInPageNotification('DLsite: 対象作品が見つかりませんでした', 'error')
  return false
}

export async function initLaunchergDownloadOnceForUrl(url: string, mark: (url: string) => void, isMarked: (url: string) => boolean): Promise<void> {
  if (isMarked(url))
    return
  const p = parseLaunchergParam()
  if (!p || p.type !== 'download')
    return
  mark(url)
  log.info('Launcherg download param detected - DLsite flow start')

  try {
    const storeId = p.value.game.storeId
    if (!storeId)
      return

    if (isDetailPage()) {
      await clickZipDownloadOnDetail(storeId)
    }
    else {
      const navigated = await performNavigateToDetailByStoreId(storeId)
      if (navigated) {
        await waitForPageLoad(800)
        await clickZipDownloadOnDetail(storeId)
      }
    }
  }
  finally {
    closeCurrentTab()
  }
}
