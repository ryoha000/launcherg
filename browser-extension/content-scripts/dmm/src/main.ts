import {
  addNotificationStyles,
  injectPageScript,
  logger,
  showInPageNotification,
} from '@launcherg/shared'
import {
  DMM_LIBRARY_SCRIPT_ID,
  DMM_LIBRARY_SCRIPT_PATH,
} from './api'
import { initLaunchergDownloadOnceForUrl } from './download'
import { processPacks, syncDmmGames } from './orchestrator'
import { createDmmRuntime } from './runtime'

const log = logger('dmm-content-script')
const downloadTriggeredForUrl = new Set<string>()
const isMarked = (url: string) => downloadTriggeredForUrl.has(url)
const mark = (url: string) => void downloadTriggeredForUrl.add(url)

const runtime = createDmmRuntime({
  initialUrl: window.location.href,
  processPacks,
  syncDmmGames,
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
      void initLaunchergDownloadOnceForUrl(nextUrl, mark, isMarked)
    })
    observer.observe(document.body, { childList: true, subtree: true })
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
  injectPageScript(chrome.runtime.getURL(DMM_LIBRARY_SCRIPT_PATH), DMM_LIBRARY_SCRIPT_ID)
  setupPageChangeObserver()
  setTimeout(() => {
    void initLaunchergDownloadOnceForUrl(window.location.href, mark, isMarked)
  }, 1000)
}

main()

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === 'manual_sync_request' || message?.type === 'periodic_sync_check') {
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
