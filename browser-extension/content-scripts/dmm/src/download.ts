import { logger, setDownloadIntent, showInPageNotification, waitForPageLoad } from '@launcherg/shared'
import { collectDownloadLinks } from './download-modal'
import { findDetailItemIdForStoreId } from './pack-helpers'
import { findChildGameImage } from './pack-parser'

const log = logger('dmm-download')

export interface LaunchergDownloadParam {
  type: 'download'
  value: {
    game: { storeId: string, category: string, subcategory: string }
    parentPack?: { storeId: string, category: string, subcategory: string }
  }
}

export function parseLaunchergParam(): LaunchergDownloadParam | null {
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
    ) {
      return parsed as LaunchergDownloadParam
    }
    return null
  }
  catch (e) {
    log.debug('Failed to parse launcherg param', e)
    return null
  }
}

// 現在のタブを閉じる（background 経由）
export function closeCurrentTab(): void {
  try {
    chrome.runtime.sendMessage({ type: 'close_current_tab' })
  }
  catch {}
}

export async function performDownloadByStoreId(storeId: string): Promise<void> {
  await waitForPageLoad(2000)
  const id = findDetailItemIdForStoreId(storeId)
  if (!id) {
    showInPageNotification('DMM: 対象ゲームが見つかりませんでした', 'error')
    return
  }
  const container = document.getElementById(id)
  if (!container) {
    showInPageNotification('DMM: 対象要素が見つかりませんでした', 'error')
    return
  }
  const img = container.querySelector('img') as HTMLImageElement | null
  if (!img) {
    showInPageNotification('DMM: クリック対象の画像が見つかりませんでした', 'error')
    return
  }
  try {
    img.scrollIntoView({ block: 'center' })
  }
  catch {}
  await new Promise(r => setTimeout(r, 200))
  img.click()
  showInPageNotification('DMM: ダウンロードを開始しました', 'success')
}

export async function initLaunchergDownloadOnceForUrl(url: string, mark: (url: string) => void, isMarked: (url: string) => boolean): Promise<void> {
  if (isMarked(url))
    return
  const p = parseLaunchergParam()
  if (!p || p.type !== 'download')
    return
  mark(url)
  log.info('Launcherg download param detected - triggering click')
  try {
    // 親パックが指定されている場合は、親の img をクリック → パックモーダル内から該当ゲーム img をクリック
    if (p.value.parentPack) {
      await performDownloadByStoreId(p.value.parentPack.storeId)
      await waitForPageLoad(800)
      const childImg = findChildGameImage(document, p.value.game.storeId)
      if (childImg) {
        try {
          childImg.scrollIntoView({ block: 'center' })
        }
        catch {}
        await new Promise(r => setTimeout(r, 200))
        childImg.click()
      }
      // 子ゲームのモーダルが開いた後、リンク件数を算出し intent を保存してからクリック
      await waitForPageLoad(800)
      const links = collectDownloadLinks(document)
      try {
        await setDownloadIntent(p.value.game.storeId, {
          store: 'DMM',
          game: p.value.game,
          parentPack: p.value.parentPack,
          expected: links.length,
          completed: 0,
          startedAt: Date.now(),
        })
      }
      catch {}
      log.info('ダウンロードリンクをクリック', { links })
      for (const a of links) {
        try {
          a.click()
          await new Promise(r => setTimeout(r, 1000))
        }
        catch {}
      }
    }
    else {
      await performDownloadByStoreId(p.value.game.storeId)
      // モーダル待機→リンク件数を算出し intent を保存してからクリック
      await waitForPageLoad(800)
      const links = collectDownloadLinks(document)
      try {
        await setDownloadIntent(p.value.game.storeId, {
          store: 'DMM',
          game: p.value.game,
          parentPack: undefined,
          expected: links.length,
          completed: 0,
          startedAt: Date.now(),
        })
      }
      catch {}
      log.info('ダウンロードリンクをクリック', { links })
      for (const a of links) {
        try {
          a.click()
          await new Promise(r => setTimeout(r, 1000))
        }
        catch {}
      }
    }
  }
  finally {
    closeCurrentTab()
  }
}
