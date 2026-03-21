import type { Page, Worker } from '@playwright/test'
import { loginToDmm } from './auth'
import { getServiceWorker, setupDownloadsSpy, setupSendNativeMessageSpy } from './extension'
import { test as base } from './fixtures'

interface WorkerFixtures {
  authenticatedDmmPage: Page
  dmmServiceWorker: Worker
}

export const test = base.extend<object, WorkerFixtures>({
  dmmServiceWorker: [
    async ({ persistentContext }, use) => {
      const sw = await getServiceWorker(persistentContext)
      await setupSendNativeMessageSpy(sw)
      await setupDownloadsSpy(sw)
      await use(sw)
    },
    { scope: 'worker' },
  ],
  authenticatedDmmPage: [
    async ({ persistentContext, dmmServiceWorker: _dmmServiceWorker }, use) => {
      const page = await persistentContext.newPage()
      await loginToDmm(page)
      await use(page)
      await page.close()
    },
    { scope: 'worker' },
  ],
})

export { expect } from './fixtures'
