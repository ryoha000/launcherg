import process from 'node:process'
import type { Page } from '@playwright/test'
import type { NativeMessageTs } from '../../shared/src/typeshare/native-messaging'
import {
  getNativeMessageCalls,
  waitForNativeMessageCall,
} from './helpers/extension'
import { expect, test } from './helpers/dmm-fixtures'

// 所持ゲームの storeId は環境変数で指定（例: \"vsat_0158\"）
// 未指定の場合は呼び出し自体の存在のみ確認する
const EXPECTED_STORE_ID = process.env.DMM_EXPECTED_STORE_ID ?? ''

async function dumpPageState(page: Page): Promise<void> {
  const state = await page.evaluate(() => {
    const root = document.querySelector('#mylibrary')
    const productList = document.querySelector('.productList')
    const images = Array.from((productList || root || document).querySelectorAll('img'))
      .slice(0, 20)
      .map(img => ({
        src: img.getAttribute('src'),
        alt: img.getAttribute('alt'),
      }))

    return {
      url: location.href,
      title: document.title,
      bodyText: document.body.textContent?.slice(0, 1000) ?? '',
      rootExists: !!root,
      productListExists: !!productList,
      imageCount: images.length,
      images,
      productListHtml: productList?.outerHTML.slice(0, 2000) ?? null,
    }
  })

  console.warn('DMM page dump:', JSON.stringify(state, null, 2))
}

// ---------------------------------------------------------------------------
// テストスイート
// ---------------------------------------------------------------------------
test.describe('DMM content-script 注入・NativeMessage 送信確認', () => {
  test.describe.configure({ timeout: 90_000 })

  // -------------------------------------------------------------------------
  // シナリオ 1: sendNativeMessage が少なくとも1回呼ばれること
  // -------------------------------------------------------------------------
  test('マイライブラリ表示後に chrome.runtime.sendNativeMessage が呼ばれる', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    try {
      // content-script が抽出・同期を完了するまで待機（最大30秒）
      const calls = await waitForNativeMessageCall(
        dmmServiceWorker,
        cs => cs.length > 0,
        30_000,
      )

      expect(calls.length).toBeGreaterThan(0)
    }
    catch (error) {
      await dumpPageState(authenticatedDmmPage)
      throw error
    }
  })

  // -------------------------------------------------------------------------
  // シナリオ 2: SyncDmmGames または GetDmmOmitWorks が送信されること
  // -------------------------------------------------------------------------
  test('NativeMessage に SyncDmmGames または GetDmmOmitWorks が含まれる', async ({ dmmServiceWorker }) => {
    const calls = await getNativeMessageCalls(dmmServiceWorker)

    const relevantCalls = calls.filter((c) => {
      const msg = c.message as NativeMessageTs
      return (
        msg?.message?.case === 'SyncDmmGames'
        || msg?.message?.case === 'GetDmmOmitWorks'
      )
    })

    expect(relevantCalls.length).toBeGreaterThan(0)
  })

  // -------------------------------------------------------------------------
  // シナリオ 3: 期待する storeId を持つ作品が現行 DOM から抽出可能であること
  // -------------------------------------------------------------------------
  test('期待する storeId を持つ作品がマイライブラリ DOM に存在する', async ({ authenticatedDmmPage }) => {
    if (!EXPECTED_STORE_ID) {
      test.skip(true, 'DMM_EXPECTED_STORE_ID が未設定のためスキップ')
      return
    }

    const found = await authenticatedDmmPage.evaluate((expectedStoreId) => {
      return Array.from(document.querySelectorAll<HTMLImageElement>('#mylibrary .productList img'))
        .some(img => img.getAttribute('src')?.includes(`/${expectedStoreId}/`))
    }, EXPECTED_STORE_ID)

    expect(
      found,
      `storeId "${EXPECTED_STORE_ID}" を含む画像 URL がマイライブラリ DOM に見つかりませんでした。`,
    ).toBe(true)
  })
})
