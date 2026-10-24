import type { IntegrationDiagnosticEventTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { sanitizeDiagnosticDetail } from '@launcherg/shared'

const PENDING_KEY = 'integration_diagnostics_pending'
export const DIAGNOSTICS_RETRY_ALARM = 'integration_diagnostics_retry'

export interface IntegrationDiagnostics {
  record: (event: IntegrationDiagnosticEventTs) => Promise<void>
  flush: () => Promise<void>
}

export function createIntegrationDiagnostics(context: HandlerContext): IntegrationDiagnostics {
  let tail = Promise.resolve()
  let timer: ReturnType<typeof setTimeout> | undefined
  const serialize = (action: () => Promise<void>) => {
    const current = tail.catch(() => {}).then(action)
    tail = current.catch(() => {})
    return current
  }
  const schedule = () => {
    if (timer)
      return
    timer = setTimeout(() => {
      timer = undefined
      void flush()
    }, 250)
  }
  async function flush(): Promise<void> {
    await serialize(async () => {
      const stored = await context.browser.storage.get([PENDING_KEY])
      const pending: IntegrationDiagnosticEventTs[] = stored[PENDING_KEY] || []
      if (!pending.length)
        return
      const batch = pending.slice(0, 100)
      const response = await context.nativeMessenger.sendJson({
        request_id: context.idGenerator.generate(),
        message: { case: 'RecordDiagnostics', value: { events: batch } },
      })
      if (!response?.success || response.response?.case !== 'RecordDiagnosticsResult')
        throw new Error(response?.error || '診断ログの保存に失敗しました')
      await context.browser.storage.set({ [PENDING_KEY]: pending.slice(batch.length), integration_diagnostics_error: '' })
      if (pending.length > batch.length)
        schedule()
    }).catch(async (error: unknown) => {
      await context.browser.storage.set({ integration_diagnostics_error: sanitizeDiagnosticDetail(String(error)) }).catch(() => {})
      await context.browser.alarms.create(DIAGNOSTICS_RETRY_ALARM, { delayInMinutes: 1 })
    })
  }
  return {
    flush,
    async record(event) {
      if (!['dlsite', 'dmm', 'extension'].includes(event.store)
        || !['auto', 'manual', 'periodic', 'background'].includes(event.trigger)
        || !['library', 'work', 'other', 'background'].includes(event.page_kind)
        || !/^[a-z_]{1,64}$/.test(event.stage)) {
        return
      }
      await serialize(async () => {
        const stored = await context.browser.storage.get([PENDING_KEY])
        const pending: IntegrationDiagnosticEventTs[] = stored[PENDING_KEY] || []
        const clean: IntegrationDiagnosticEventTs = {
          timestamp: new Date().toISOString(),
          store: event.store,
          stage: event.stage,
          trigger: event.trigger,
          page_kind: event.page_kind,
          count: typeof event.count === 'number' && Number.isSafeInteger(event.count) && event.count >= 0 ? Math.min(event.count, 0xFFFFFFFF) : undefined,
          detail: sanitizeDiagnosticDetail(String(event.detail || '')),
        }
        await context.browser.storage.set({ [PENDING_KEY]: [...pending, clean].slice(-200) })
      })
      schedule()
    },
  }
}
