import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from '@playwright/test'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
export const extensionPath = path.resolve(__dirname, 'dist')

export default defineConfig({
  testDir: './tests/e2e',
  // 並列実行はログイン状態の競合を防ぐため無効化
  workers: 1,
  retries: 0,
  timeout: 30_000,
  projects: [
    {
      name: 'chromium-with-extension',
    },
  ],
})
