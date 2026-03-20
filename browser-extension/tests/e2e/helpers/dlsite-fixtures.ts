import type { Page, Worker } from '@playwright/test'
import { loginToDlsite } from './auth'
import { getServiceWorker, setupSendNativeMessageSpy } from './extension'
import { test as base } from './fixtures'

interface WorkerFixtures {
  authenticatedDlsitePage: Page
  dlsiteServiceWorker: Worker
}

export const test = base.extend<{}, WorkerFixtures>({
  dlsiteServiceWorker: [
    async ({ persistentContext }, use) => {
      const sw = await getServiceWorker(persistentContext)
      await setupSendNativeMessageSpy(sw)
      await use(sw)
    },
    { scope: 'worker' },
  ],
  authenticatedDlsitePage: [
    async ({ persistentContext, dlsiteServiceWorker: _dlsiteServiceWorker }, use) => {
      const page = await persistentContext.newPage()
      await loginToDlsite(page)
      await use(page)
      await page.close()
    },
    { scope: 'worker' },
  ],
})

export { expect } from './fixtures'
