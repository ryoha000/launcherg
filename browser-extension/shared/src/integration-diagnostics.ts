import type { IntegrationDiagnosticEventTs } from './typeshare/native-messaging'

export type DiagnosticStore = 'dlsite' | 'dmm' | 'extension'
export type SyncTrigger = 'auto' | 'manual' | 'periodic' | 'background'
export type DiagnosticProgress = Pick<IntegrationDiagnosticEventTs, 'stage'> & Partial<Pick<IntegrationDiagnosticEventTs, 'count' | 'detail'>>

export function diagnosticPageKind(url: string): string {
  try {
    const parsed = new URL(url)
    const path = parsed.pathname
    if (path.includes('/library') || (parsed.hostname === 'play.dlsite.com' && path === '/'))
      return 'library'
    if (path.includes('/work/'))
      return 'work'
  }
  catch {}
  return 'other'
}

export function sanitizeDiagnosticDetail(value: string): string {
  // Error messages may contain URLs with tokens. Never persist those URLs.
  return value.replace(/https?:\/\/[^\s"'<>]+/gi, '[URL]').replace(/[\r\n\t]+/g, ' ').slice(0, 500)
}

export function reportIntegrationDiagnostic(store: DiagnosticStore, progress: DiagnosticProgress, trigger: SyncTrigger = 'auto'): void {
  const event: IntegrationDiagnosticEventTs = {
    timestamp: new Date().toISOString(),
    store,
    stage: progress.stage,
    trigger,
    page_kind: diagnosticPageKind(window.location.href),
    count: progress.count,
    detail: sanitizeDiagnosticDetail(progress.detail || ''),
  }
  try {
    chrome.runtime.sendMessage({ type: 'integration_diagnostic', event }, () => {
      void chrome.runtime.lastError
    })
  }
  catch { /* Diagnostics never interrupt sync. */ }
}
