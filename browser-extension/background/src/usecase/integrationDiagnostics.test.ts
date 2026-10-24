import type { IntegrationDiagnosticEventTs, NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { buildTestContext } from '../../test/helpers/context'
import { createIntegrationDiagnostics } from './integrationDiagnostics'

afterEach(() => vi.useRealTimers())

describe('自動同期の診断保存', () => {
  it('host保存失敗時は保持し再起動後に再送し、URLの秘密情報を保存しない', async () => {
    vi.useFakeTimers()
    const state: Record<string, unknown> = {}
    const context = buildTestContext()
    context.browser.storage.get = async () => state
    context.browser.storage.set = async (values) => {
      Object.assign(state, values)
    }
    const send = vi.fn<() => Promise<NativeResponseTs | null>>()
      .mockRejectedValueOnce(new Error('Host unavailable'))
      .mockResolvedValue({ success: true, error: '', request_id: 'r', response: { case: 'RecordDiagnosticsResult', value: { recorded: 1 } } })
    context.nativeMessenger.sendJson = send
    const event: IntegrationDiagnosticEventTs = { timestamp: '', store: 'dlsite', stage: 'sync_failed', trigger: 'auto', page_kind: 'library', detail: 'fetch https://example.test/?token=secret failed' }
    const diagnostics = createIntegrationDiagnostics(context)
    await diagnostics.record(event)
    await diagnostics.flush()
    expect(state.integration_diagnostics_pending).toHaveLength(1)
    expect(JSON.stringify(state)).not.toContain('secret')
    await createIntegrationDiagnostics(context).flush()
    expect(state.integration_diagnostics_pending).toEqual([])
    expect(send).toHaveBeenCalledTimes(2)
  })
})
