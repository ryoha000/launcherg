import type { BrowserContext } from '@playwright/test'
/**
 * Playwright カスタムフィクスチャ
 *
 * chrome 拡張機能のテストには `chromium.launchPersistentContext` が必要。
 * 通常の `browser.newContext()` では Service Worker が起動しないため、
 * persistentContext を使ってブラウザを起動し直す。
 */
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { test as base, chromium } from '@playwright/test'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const extensionPath = path.resolve(__dirname, '../../../dist')

/** ワーカー単位で共有するフィクスチャの型定義 */
interface WorkerFixtures {
  persistentContext: BrowserContext
}

/** テスト全体で共有する persistentContext（Service Worker 込み） */
export const test = base.extend<object, WorkerFixtures>({
  persistentContext: [
    // Playwright の fixture シグネチャに合わせて空の分割代入を使う。
    // eslint-disable-next-line no-empty-pattern
    async ({}, use) => {
      const context = await chromium.launchPersistentContext('', {
        headless: false,
        args: [
          `--disable-extensions-except=${extensionPath}`,
          `--load-extension=${extensionPath}`,
        ],
      })
      await use(context)
      await context.close()
    },
    { scope: 'worker' },
  ],
})

export { expect } from '@playwright/test'
