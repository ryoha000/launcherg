import {
  addNotificationStyles,
  diagnosticPageKind,
  logger,
  reportIntegrationDiagnostic,
  showInPageNotification,
} from '@launcherg/shared'
import { initLaunchergDownloadOnceForUrl } from './download'
import { processPacks, syncDmmGames } from './orchestrator'
import { createDmmRuntime } from './runtime'

const log = logger('dmm-content-script')
const CONTENT_SCRIPT_FLAG = '__launchergDmmContentScriptInstalled__'
const CONTENT_SCRIPT_MARKER = 'data-launcherg-dmm-content-script-installed'
const downloadTriggeredForUrl = new Set<string>()
const isMarked = (url: string) => downloadTriggeredForUrl.has(url)
const mark = (url: string) => void downloadTriggeredForUrl.add(url)

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

const runtime = createDmmRuntime({
  initialUrl: window.location.href,
  processPacks,
  syncDmmGames,
  showErrorNotification: message => showInPageNotification(message, 'error'),
  onProgress: (progress, trigger) => {
    if (progress.stage === 'payload_received')
      receivedPayload = true
    reportIntegrationDiagnostic('dmm', progress, trigger)
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
  ensureContentScriptMarker()
  addNotificationStyles()
  window.addEventListener('message', runtime.handleHookMessage)
  reportIntegrationDiagnostic('dmm', { stage: 'content_started' })
  window.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (event.source !== window || typeof event.data !== 'object' || event.data === null)
      return
    const data = event.data as { source?: string, type?: string, stage?: string, detail?: string }
    if (data.source !== 'launcherg' || data.type !== 'launcherg:dmm-hook-status')
      return
    if (data.stage === 'hook_ready')
      hookReady = true
    if (['hook_ready', 'api_requested', 'api_response', 'api_invalid', 'api_failed'].includes(data.stage || ''))
      reportIntegrationDiagnostic('dmm', { stage: data.stage!, detail: typeof data.detail === 'string' ? data.detail : '' })
  })
  window.postMessage({ source: 'launcherg', type: 'launcherg:dmm-content-ready' }, window.location.origin)
  setTimeout(() => {
    if (!hookReady)
      reportIntegrationDiagnostic('dmm', { stage: 'hook_missing', detail: 'ページの通信監視を初期化できませんでした' })
  }, 3000)
  setTimeout(() => {
    if (!receivedPayload && diagnosticPageKind(window.location.href) === 'library')
      reportIntegrationDiagnostic('dmm', { stage: 'payload_waiting', detail: '購入一覧のAPIレスポンスをまだ受信していません' })
  }, 15000)
  setupPageChangeObserver()
  setTimeout(() => {
    void initLaunchergDownloadOnceForUrl(window.location.href, mark, isMarked)
  }, 1000)
}

main()

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === 'manual_sync_request' || message?.type === 'periodic_sync_check') {
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
