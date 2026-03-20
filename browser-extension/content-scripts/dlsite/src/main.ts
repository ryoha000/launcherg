import {
  addNotificationStyles,
  injectPageScript,
  logger,
  showInPageNotification,
} from '@launcherg/shared'
import {
  DLSITE_WORKS_SCRIPT_ID,
  DLSITE_WORKS_SCRIPT_PATH,
} from './api'
import { processGames } from './data-processor'
import { initLaunchergDownloadOnceForUrl } from './download'
import { syncDlsiteGames } from './orchestrator'
import { createDlsiteRuntime } from './runtime'

const log = logger('dlsite-content-script')

// ダウンロード起動の一度きり実行制御
const processedUrlSet = new Set<string>()
function markProcessed(url: string): void {
  processedUrlSet.add(url)
}
function isProcessed(url: string): boolean {
  return processedUrlSet.has(url)
}

const runtime = createDlsiteRuntime({
  initialUrl: window.location.href,
  processGames,
  syncDlsiteGames,
  showErrorNotification: message => showInPageNotification(message, 'error'),
})

function setupPageChangeObserver(): void {
  let currentUrl = window.location.href
  const observe = () => {
    const observer = new MutationObserver(() => {
      const nextUrl = window.location.href
      if (nextUrl === currentUrl)
        return

      currentUrl = nextUrl
      runtime.handleUrlChange(nextUrl)
      void initLaunchergDownloadOnceForUrl(nextUrl, markProcessed, isProcessed)
    })

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    })
  }

  if (document.body) {
    observe()
    return
  }

  window.addEventListener('DOMContentLoaded', observe, { once: true })
}

function main(): void {
  log.info('Script loaded')

  addNotificationStyles()
  window.addEventListener('message', runtime.handleHookMessage)
  injectPageScript(chrome.runtime.getURL(DLSITE_WORKS_SCRIPT_PATH), DLSITE_WORKS_SCRIPT_ID)

  setupPageChangeObserver()

  setTimeout(() => {
    void initLaunchergDownloadOnceForUrl(window.location.href, markProcessed, isProcessed)
  }, 1000)
}

main()

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (
    message?.type === 'manual_sync_request'
    || message?.type === 'periodic_sync_check'
  ) {
    void runtime.syncLatest()
      .then(result => sendResponse(result))
      .catch((err: unknown) => {
        const errorMessage = err instanceof Error ? err.message : String(err)
        sendResponse({ success: false, error: errorMessage })
      })
    return true
  }
  return undefined
})
