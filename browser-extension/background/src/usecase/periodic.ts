import type { Browser, BrowserTab } from '../shared/types'
import { logger } from '@launcherg/shared'

const log = logger('background:periodic')

export async function performPeriodicSync(browser: Browser): Promise<void> {
  log.info('Performing periodic sync check')

  const tabs = await browser.tabs.query({
    url: ['https://dlsoft.dmm.co.jp/*', 'https://play.dlsite.com/*'],
  })

  for (const tab of tabs) {
    if (!tab.id)
      continue
    await sendMessageToTabWithInjection(browser, tab, { type: 'periodic_sync_check' })
  }
}

async function sendMessageToTabWithInjection(
  browser: Browser,
  tab: BrowserTab,
  message: unknown,
): Promise<void> {
  if (!tab.id || !tab.url)
    return
  try {
    await browser.tabs.sendMessage(tab.id, message as any)
  }
  catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    if (!/Receiving end does not exist/i.test(errorMessage)) {
      log.warn('sendMessage failed (non-receiver error):', errorMessage)
      return
    }
    const files: string[] = []
    if (tab.url.includes('dlsoft.dmm.co.jp'))
      files.push('content-scripts/dmm-extractor.js')
    if (tab.url.includes('play.dlsite.com'))
      files.push('content-scripts/dlsite-extractor.js')

    if (files.length === 0)
      return

    try {
      await browser.scripting.executeScript(tab.id, files)
      await browser.tabs.sendMessage(tab.id, message as any)
    }
    catch (injectErr) {
      log.warn('Failed to inject/retry sendMessage:', injectErr)
    }
  }
}
