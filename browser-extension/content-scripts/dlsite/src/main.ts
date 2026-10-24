import {
  addNotificationStyles,
  diagnosticPageKind,
  logger,
  reportIntegrationDiagnostic,
  showInPageNotification,
} from '@launcherg/shared'
import { processGames } from './data-processor'
import { initLaunchergDownloadOnceForUrl } from './download'
import { syncDlsiteGames } from './orchestrator'
import { createDlsiteRuntime } from './runtime'

const log = logger('dlsite-content-script')
const CONTENT_SCRIPT_FLAG = '__launchergDlsiteContentScriptInstalled__'
const CONTENT_SCRIPT_MARKER = 'data-launcherg-dlsite-content-script-installed'

// ダウンロード起動の一度きり実行制御
const processedUrlSet = new Set<string>()
function markProcessed(url: string): void {
  processedUrlSet.add(url)
}
function isProcessed(url: string): boolean {
  return processedUrlSet.has(url)
}

function setContentScriptMarker(): void {
  ;(window as typeof window & { [CONTENT_SCRIPT_FLAG]?: boolean })[CONTENT_SCRIPT_FLAG] = true
  document.documentElement?.setAttribute(CONTENT_SCRIPT_MARKER, 'true')
}

function ensureContentScriptMarker(): void {
  if (document.documentElement) {
    setContentScriptMarker()
    return
  }

  window.addEventListener('DOMContentLoaded', setContentScriptMarker, { once: true })
}

let receivedPayload = false
let hookReady = false

const runtime = createDlsiteRuntime({
  initialUrl: window.location.href,
  processGames,
  syncDlsiteGames,
  showErrorNotification: message => showInPageNotification(message, 'error'),
  onProgress: (progress, trigger) => {
    if (progress.stage === 'payload_received')
      receivedPayload = true
    reportIntegrationDiagnostic('dlsite', progress, trigger)
  },
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
      window.postMessage({ source: 'launcherg', type: 'launcherg:dlsite-library-ready' }, window.location.origin)
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
  ensureContentScriptMarker()

  addNotificationStyles()
  window.addEventListener('message', runtime.handleHookMessage)
  reportIntegrationDiagnostic('dlsite', { stage: 'content_started' })
  window.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (event.source !== window || typeof event.data !== 'object' || event.data === null)
      return
    const data = event.data as { source?: string, type?: string, stage?: string, detail?: string }
    if (data.source !== 'launcherg' || data.type !== 'launcherg:dlsite-hook-status')
      return
    if (data.stage === 'hook_ready')
      hookReady = true
    if (['hook_ready', 'api_requested', 'api_response', 'api_invalid', 'api_failed', 'cache_opened', 'cache_reading', 'cache_read', 'cache_failed'].includes(data.stage || ''))
      reportIntegrationDiagnostic('dlsite', { stage: data.stage!, detail: typeof data.detail === 'string' ? data.detail : '' })
  })
  window.postMessage({ source: 'launcherg', type: 'launcherg:dlsite-content-ready' }, window.location.origin)
  setTimeout(() => {
    if (!hookReady)
      reportIntegrationDiagnostic('dlsite', { stage: 'hook_missing', detail: 'ページの通信監視を初期化できませんでした' })
  }, 3000)
  setTimeout(() => {
    if (!receivedPayload && diagnosticPageKind(window.location.href) === 'library')
      reportIntegrationDiagnostic('dlsite', { stage: 'payload_waiting', detail: '購入一覧のAPIレスポンスをまだ受信していません' })
  }, 15000)

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
    void runtime.syncLatest(message.type === 'periodic_sync_check' ? 'periodic' : 'manual')
      .then(result => sendResponse(result))
      .catch((err: unknown) => {
        const errorMessage = err instanceof Error ? err.message : String(err)
        sendResponse({ success: false, error: errorMessage })
      })
    return true
  }
  return undefined
})
