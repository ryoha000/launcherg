import type { Page } from '@playwright/test'
import type { NativeMessageTs } from '../../shared/src/typeshare/native-messaging'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import process from 'node:process'
import { expect, test } from './helpers/dmm-fixtures'
import {
  clearDownloadIntents,
  emitDownloadComplete,
  getDownloadIntents,
  resetDownloadSpy,
  resetNativeMessageSpy,
  waitForDownloadCalls,
  waitForDownloadIntent,
  waitForNativeMessageCall,
} from './helpers/extension'

// 所持ゲームの storeId は環境変数で指定（例: \"vsat_0158\"）
// 未指定の場合は呼び出し自体の存在のみ確認する
const EXPECTED_STORE_ID = process.env.DMM_EXPECTED_STORE_ID ?? ''
const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const sampleDmmNotPack = JSON.parse(readFileSync(resolve(__dirname, '../unit/data/sample_dmm_not_pack.json'), 'utf-8')) as {
  body?: {
    productDetail?: {
      download?: {
        combinedFileUrl?: string | null
        splitFileUrlArray?: Array<string | null> | null
        singleFileUrl?: string | null
      } | null
    } | null
  } | null
}

const sampleDmmPack = JSON.parse(readFileSync(resolve(__dirname, '../unit/data/sample_dmm_pack.json'), 'utf-8')) as {
  body?: {
    childProducts?: Array<{
      product?: { productId?: string | null } | null
      download?: {
        combinedFileUrl?: string | null
        splitFileUrlArray?: Array<string | null> | null
        singleFileUrl?: string | null
      } | null
    }> | null
  } | null
}

const sampleDmmLibrary = JSON.parse(readFileSync(resolve(__dirname, '../unit/data/sample_dmm.json'), 'utf-8'))

function extractExpectedUrls(download: {
  combinedFileUrl?: string | null
  splitFileUrlArray?: Array<string | null> | null
  singleFileUrl?: string | null
} | null | undefined): string[] {
  const urls = [
    download?.combinedFileUrl ?? null,
    ...(download?.splitFileUrlArray ?? []),
    download?.singleFileUrl ?? null,
  ]
    .filter((url): url is string => typeof url === 'string' && url.length > 0)
    .map(url => new URL(url, 'https://dlsoft.dmm.co.jp').toString())

  if (urls.length === 0)
    throw new Error('サンプル JSON から期待URLを組み立てられませんでした')

  return urls
}

const notPackExpectedUrls = extractExpectedUrls(sampleDmmNotPack.body?.productDetail?.download)
const packChild = (sampleDmmPack.body?.childProducts ?? []).find(child => child.product?.productId === 'views_0571')
if (!packChild)
  throw new Error('sample_dmm_pack.json から views_0571 が見つかりませんでした')
const packExpectedUrls = extractExpectedUrls(packChild.download)

async function openLaunchergDownloadPage(
  page: Page,
  payload: Record<string, unknown>,
): Promise<Page> {
  const url = new URL('https://dlsoft.dmm.co.jp/library/')
  url.hash = new URLSearchParams({ launcherg: JSON.stringify(payload) }).toString()

  const downloadPage = await page.context().newPage()
  try {
    await downloadPage.goto(url.toString(), { waitUntil: 'commit' })
  }
  catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    if (!downloadPage.isClosed() || !message.includes('Target page, context or browser has been closed'))
      throw error
  }
  return downloadPage
}

async function triggerLibraryApi(page: Page, payload: unknown, pageNo = 1): Promise<void> {
  const requestUrl = `https://dlsoft.dmm.co.jp/ajax/v1/library?page=${pageNo}`
  await page.context().route(requestUrl, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(payload),
    })
  })

  try {
    await page.evaluate(async (url) => {
      await fetch(url, { credentials: 'include' })
    }, requestUrl)
  }
  finally {
    await page.context().unroute(requestUrl)
  }
}

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

  test.beforeEach(async ({ dmmServiceWorker }) => {
    await clearDownloadIntents(dmmServiceWorker)
    await resetDownloadSpy(dmmServiceWorker)
    await resetNativeMessageSpy(dmmServiceWorker)
  })

  // -------------------------------------------------------------------------
  // シナリオ 1: sendNativeMessage が少なくとも1回呼ばれること
  // -------------------------------------------------------------------------
  test('マイライブラリ表示後に chrome.runtime.sendNativeMessage が呼ばれる', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    try {
      await triggerLibraryApi(authenticatedDmmPage, sampleDmmLibrary)

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
  // シナリオ 2: SyncDmmGames が送信されること
  // -------------------------------------------------------------------------
  test('NativeMessage に SyncDmmGames が含まれる', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    await triggerLibraryApi(authenticatedDmmPage, sampleDmmLibrary, 2)

    const calls = await waitForNativeMessageCall(
      dmmServiceWorker,
      cs => cs.length > 0,
      30_000,
    )

    const relevantCalls = calls.filter((c) => {
      const msg = c.message as NativeMessageTs
      return msg?.message?.case === 'SyncDmmGames'
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

  // -------------------------------------------------------------------------
  // シナリオ 4: launcherg 付きのダウンロードURLから intent が保存される
  // -------------------------------------------------------------------------
  test('非パック作品は detail/single を使って direct download を開始する', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    let singleRequestCount = 0
    await authenticatedDmmPage.context().route('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/single/?productId=hobc_0157', async (route) => {
      singleRequestCount += 1
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(sampleDmmNotPack),
      })
    })

    try {
      const downloadPage = await openLaunchergDownloadPage(authenticatedDmmPage, {
        type: 'download',
        value: {
          game: { storeId: 'hobc_0157', category: 'digital', subcategory: 'pcgame' },
        },
      })

      const calls = await waitForDownloadCalls(dmmServiceWorker, cs => cs.length === notPackExpectedUrls.length, 45_000)
      const intent = await waitForDownloadIntent(dmmServiceWorker, 'hobc_0157', 10_000) as {
        store?: string
        game?: { storeId?: string }
        expected?: number
      }

      expect(intent.store).toBe('DMM')
      expect(intent.game?.storeId).toBe('hobc_0157')
      expect(intent.expected).toBe(notPackExpectedUrls.length)
      expect(singleRequestCount).toBe(1)
      expect(calls.map(call => call.url)).toEqual(notPackExpectedUrls)

      await downloadPage.close().catch(() => {})
    }
    finally {
      await authenticatedDmmPage.context().unroute('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/single/?productId=hobc_0157')
    }
  })

  test('パック作品は detail/set から対象 child の direct download だけを開始する', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    let setRequestCount = 0
    await authenticatedDmmPage.context().route('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/set/?productId=purple_0028pack', async (route) => {
      setRequestCount += 1
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(sampleDmmPack),
      })
    })

    try {
      const downloadPage = await openLaunchergDownloadPage(authenticatedDmmPage, {
        type: 'download',
        value: {
          game: { storeId: 'views_0571', category: 'digital', subcategory: 'pcgame' },
          parentPack: { storeId: 'purple_0028pack', category: 'digital', subcategory: 'pcgame' },
        },
      })

      const intent = await waitForDownloadIntent(dmmServiceWorker, 'views_0571', 45_000) as {
        store?: string
        game?: { storeId?: string }
        parentPack?: { storeId?: string }
        expected?: number
      }
      const calls = await waitForDownloadCalls(dmmServiceWorker, cs => cs.length === packExpectedUrls.length, 45_000)

      expect(intent.store).toBe('DMM')
      expect(intent.game?.storeId).toBe('views_0571')
      expect(intent.parentPack?.storeId).toBe('purple_0028pack')
      expect(intent.expected).toBe(packExpectedUrls.length)
      expect(setRequestCount).toBe(1)
      expect(calls.map(call => call.url)).toEqual(packExpectedUrls)
      expect(calls.every(call => call.url.includes('productId=views_0571'))).toBe(true)

      await downloadPage.close().catch(() => {})
    }
    finally {
      await authenticatedDmmPage.context().unroute('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/set/?productId=purple_0028pack')
    }
  })

  test('direct download URL の productId で完了追跡できる', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    const singleFileOnly = structuredClone(sampleDmmNotPack)
    if (singleFileOnly.body?.productDetail?.download) {
      singleFileOnly.body.productDetail.download.combinedFileUrl = '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.exe&productId=hobc_0157&floor=Apcgame'
      singleFileOnly.body.productDetail.download.splitFileUrlArray = []
      singleFileOnly.body.productDetail.download.singleFileUrl = null
    }
    const expectedUrl = extractExpectedUrls(singleFileOnly.body?.productDetail?.download)[0]

    await authenticatedDmmPage.context().route('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/single/?productId=hobc_0157', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(singleFileOnly),
      })
    })

    try {
      const downloadPage = await openLaunchergDownloadPage(authenticatedDmmPage, {
        type: 'download',
        value: {
          game: { storeId: 'hobc_0157', category: 'digital', subcategory: 'pcgame' },
        },
      })

      const calls = await waitForDownloadCalls(dmmServiceWorker, cs => cs.length === 1, 45_000)
      expect(calls[0].url).toBe(expectedUrl)

      await emitDownloadComplete(dmmServiceWorker, calls[0].id, {
        url: calls[0].url,
        filename: 'hobc_0157.exe',
      })

      const nativeCalls = await waitForNativeMessageCall(
        dmmServiceWorker,
        cs => cs.some((c) => {
          const msg = c.message as NativeMessageTs
          return msg?.message?.case === 'DownloadsCompleted'
        }),
        45_000,
      )
      const completed = nativeCalls.find((c) => {
        const msg = c.message as NativeMessageTs
        return msg?.message?.case === 'DownloadsCompleted'
      })
      const message = completed?.message as NativeMessageTs

      expect(message.message.case).toBe('DownloadsCompleted')
      expect(message.message.value.intent.case).toBe('Dmm')
      expect(message.message.value.intent.value.game_store_id).toBe('hobc_0157')

      const intents = await getDownloadIntents(dmmServiceWorker)
      expect(intents.hobc_0157).toBeUndefined()

      await downloadPage.close().catch(() => {})
    }
    finally {
      await authenticatedDmmPage.context().unroute('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/single/?productId=hobc_0157')
    }
  })
})
