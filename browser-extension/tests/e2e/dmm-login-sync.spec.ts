import type { Page, Worker } from '@playwright/test'
import type { NativeMessageTs } from '../../shared/src/typeshare/native-messaging'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import { expect, test } from './helpers/dmm-fixtures'
import { navigateToMyLibrary } from './helpers/auth'
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

// 所持ゲームの storeId は環境変数で指定（例: "vsat_0158"）
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

function extractSyncedStoreIds(message: NativeMessageTs['message']): string[] {
  if (message.case !== 'SyncDmmGames')
    return []

  return message.value.games
    .map(game => game.id)
    .filter((id): id is string => typeof id === 'string' && id.length > 0)
}

async function waitForDmmSyncMessage(
  dmmServiceWorker: Worker,
  expectedStoreId: string,
): Promise<NativeMessageTs['message']> {
  const calls = await waitForNativeMessageCall(
    dmmServiceWorker,
    (nativeCalls) => {
      return nativeCalls.some((call) => {
        const msg = call.message as NativeMessageTs
        if (msg?.message?.case !== 'SyncDmmGames')
          return false
        return extractSyncedStoreIds(msg.message).includes(expectedStoreId)
      })
    },
    15_000,
  )

  const matched = calls.find((call) => {
    const msg = call.message as NativeMessageTs
    return msg?.message?.case === 'SyncDmmGames'
      && extractSyncedStoreIds(msg.message).includes(expectedStoreId)
  })

  if (!matched) {
    throw new Error(`SyncDmmGames に storeId "${expectedStoreId}" が見つかりませんでした`)
  }

  return (matched.message as NativeMessageTs).message
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

  test('マイライブラリ表示後に SyncDmmGames に期待する storeId が含まれる', async ({ authenticatedDmmPage, dmmServiceWorker }) => {
    if (!EXPECTED_STORE_ID) {
      test.skip(true, 'DMM_EXPECTED_STORE_ID が未設定のためスキップ')
      return
    }

    await navigateToMyLibrary(authenticatedDmmPage)
    const message = await waitForDmmSyncMessage(dmmServiceWorker, EXPECTED_STORE_ID)

    expect(extractSyncedStoreIds(message)).toContain(EXPECTED_STORE_ID)
  })

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
      const completedMessage = message.message.case === 'DownloadsCompleted' ? message.message : null

      expect(completedMessage?.case).toBe('DownloadsCompleted')
      expect(completedMessage?.value.intent.case).toBe('Dmm')
      expect(completedMessage?.value.intent.value.game_store_id).toBe('hobc_0157')

      const intents = await getDownloadIntents(dmmServiceWorker)
      expect(intents.hobc_0157).toBeUndefined()

      await downloadPage.close().catch(() => {})
    }
    finally {
      await authenticatedDmmPage.context().unroute('https://dlsoft.dmm.co.jp/ajax/v1/library/detail/single/?productId=hobc_0157')
    }
  })
})
