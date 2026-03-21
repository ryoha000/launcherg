import type { Page, Worker } from '@playwright/test'
import type { NativeMessageTs } from '../../shared/src/typeshare/native-messaging'
import process from 'node:process'
import { navigateToDlsiteLibrary } from './helpers/auth'
import { expect, test } from './helpers/dlsite-fixtures'
import { resetNativeMessageSpy, waitForNativeMessageCall } from './helpers/extension'

// 所持作品の storeId は環境変数で指定（例: "RJ109462"）
// 未指定の場合は呼び出し自体の存在のみ確認する
const EXPECTED_STORE_ID = process.env.DLSITE_EXPECTED_STORE_ID ?? ''

function extractSyncedStoreIds(message: NativeMessageTs['message']): string[] {
  if (message.case !== 'SyncDlsiteGames')
    return []

  return message.value.games
    .map(game => game.id)
    .filter((id): id is string => typeof id === 'string' && id.length > 0)
}

async function waitForDlsiteSyncMessage(
  dlsiteServiceWorker: Worker,
  expectedStoreId: string,
): Promise<NativeMessageTs['message']> {
  const calls = await waitForNativeMessageCall(
    dlsiteServiceWorker,
    (nativeCalls) => {
      return nativeCalls.some((call) => {
        const msg = call.message as NativeMessageTs
        if (msg?.message?.case !== 'SyncDlsiteGames')
          return false
        return extractSyncedStoreIds(msg.message).includes(expectedStoreId)
      })
    },
    15_000,
  )

  const matched = calls.find((call) => {
    const msg = call.message as NativeMessageTs
    return msg?.message?.case === 'SyncDlsiteGames'
      && extractSyncedStoreIds(msg.message).includes(expectedStoreId)
  })

  if (!matched) {
    throw new Error(`SyncDlsiteGames に storeId "${expectedStoreId}" が見つかりませんでした`)
  }

  return (matched.message as NativeMessageTs).message
}

async function waitForDlsiteInjection(page: Page): Promise<void> {
  await page.waitForFunction(() => {
    return document.documentElement?.getAttribute('data-launcherg-dlsite-content-script-installed') === 'true'
  }, undefined, { timeout: 3_000 })

  await page.waitForFunction(() => {
    return document.documentElement?.getAttribute('data-launcherg-dlsite-network-hook-installed') === 'true'
  }, undefined, { timeout: 3_000 })
}

test.describe('DLsite content-script 注入・NativeMessage 送信確認', () => {
  test.describe.configure({ timeout: 90_000 })

  test.beforeEach(async ({ dlsiteServiceWorker }) => {
    await resetNativeMessageSpy(dlsiteServiceWorker)
  })

  test('購入済み作品一覧表示後に SyncDlsiteGames に期待する storeId が含まれる', async ({ authenticatedDlsitePage, dlsiteServiceWorker }) => {
    if (!EXPECTED_STORE_ID) {
      test.skip(true, 'DLSITE_EXPECTED_STORE_ID が未設定のためスキップ')
      return
    }

    await navigateToDlsiteLibrary(authenticatedDlsitePage)
    await waitForDlsiteInjection(authenticatedDlsitePage)
    // await triggerDlsiteLibrarySync(authenticatedDlsitePage)
    const message = await waitForDlsiteSyncMessage(dlsiteServiceWorker, EXPECTED_STORE_ID)

    expect(extractSyncedStoreIds(message)).toContain(EXPECTED_STORE_ID)
  })
})
