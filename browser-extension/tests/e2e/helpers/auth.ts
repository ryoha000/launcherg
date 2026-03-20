import type { Page } from '@playwright/test'
import process from 'node:process'

const DMM_LOGIN_URL = 'https://accounts.dmm.co.jp/service/login/password/'
const DMM_MYLIBRARY_URL = 'https://dlsoft.dmm.co.jp/mylibrary/'
const DMM_AGE_CHECK_URL = 'https://www.dmm.co.jp/age_check/'
const DLSITE_LOGIN_URL = 'https://login.dlsite.com/login'
const DLSITE_LIBRARY_URL = 'https://play.dlsite.com/home/library'
const STEP_TIMEOUT_MS = 1_000
const STEP_INTERVAL_MS = 300
const NAVIGATION_TIMEOUT_MS = 5_000

async function waitBetweenSteps(page: Page): Promise<void> {
  await page.waitForTimeout(STEP_INTERVAL_MS)
}

async function passDmmAgeCheck(page: Page): Promise<void> {
  const isAgeCheckPage = () => page.url().startsWith(DMM_AGE_CHECK_URL)

  if (!isAgeCheckPage()) {
    return
  }

  await page.getByRole('link', { name: 'はい' }).click()
  await waitBetweenSteps(page)
  await page.waitForURL(url => !url.toString().startsWith(DMM_AGE_CHECK_URL), {
    timeout: NAVIGATION_TIMEOUT_MS,
  })
  await waitBetweenSteps(page)
}

async function fillFirstAvailable(page: Page, selectors: string[], value: string): Promise<void> {
  for (const selector of selectors) {
    const locator = page.locator(selector).first()
    try {
      await locator.waitFor({ state: 'visible', timeout: STEP_TIMEOUT_MS })
      await locator.fill(value)
      await waitBetweenSteps(page)
      return
    }
    catch {
      continue
    }
  }

  throw new Error(`入力欄が見つかりませんでした: ${selectors.join(', ')}`)
}

async function clickFirstAvailable(page: Page, selectors: string[]): Promise<void> {
  for (const selector of selectors) {
    const locator = page.locator(selector).first()
    try {
      if (await locator.count() === 0) {
        continue
      }
      await locator.waitFor({ state: 'visible', timeout: STEP_TIMEOUT_MS })
      await locator.click()
      await waitBetweenSteps(page)
      return
    }
    catch {
      continue
    }
  }

  throw new Error(`クリック対象が見つかりませんでした: ${selectors.join(', ')}`)
}

/**
 * DMM にログインする。
 * メール・パスワードは環境変数から取得する。
 *   DMM_EMAIL    : DMM アカウントのメールアドレス
 *   DMM_PASSWORD : DMM アカウントのパスワード
 */
export async function loginToDmm(page: Page): Promise<void> {
  const email = process.env.DMM_EMAIL
  const password = process.env.DMM_PASSWORD

  if (!email || !password) {
    throw new Error(
      'DMM_EMAIL または DMM_PASSWORD 環境変数が設定されていません。\n'
      + '実行前に以下のように設定してください:\n'
      + '  $env:DMM_EMAIL="your@email.com"\n'
      + '  $env:DMM_PASSWORD="yourpassword"',
    )
  }

  await page.goto(DMM_LOGIN_URL, { waitUntil: 'commit', timeout: NAVIGATION_TIMEOUT_MS })
  await waitBetweenSteps(page)

  // メールアドレス入力
  await page.locator('input[name="login_id"]').fill(email)
  await waitBetweenSteps(page)
  // パスワード入力
  await page.locator('input[name="password"]').fill(password)
  await waitBetweenSteps(page)
  // ログインボタンクリック
  await page.getByRole('button', { name: 'ログイン' }).click()
  await waitBetweenSteps(page)

  // ログイン後のリダイレクトを待つ
  await page.waitForURL(url => !url.toString().includes('accounts.dmm.co.jp'), {
    timeout: NAVIGATION_TIMEOUT_MS,
  })
  await waitBetweenSteps(page)
  await passDmmAgeCheck(page)
}

/**
 * DMM マイライブラリページへ遷移する。
 * コンテンツの読み込みが終わるまで待機する。
 */
export async function navigateToMyLibrary(page: Page): Promise<void> {
  await page.goto(DMM_MYLIBRARY_URL, { waitUntil: 'domcontentloaded', timeout: NAVIGATION_TIMEOUT_MS })
  await waitBetweenSteps(page)
  await passDmmAgeCheck(page)

  // マイライブラリのルート要素が表示されるまで待機
  await page.locator('#mylibrary').waitFor({ state: 'visible', timeout: STEP_TIMEOUT_MS })
  await waitBetweenSteps(page)
}

/**
 * DLsite にログインする。
 * メール・パスワードは環境変数から取得する。
 *   DLSITE_EMAIL    : DLsite アカウントのメールアドレス
 *   DLSITE_PASSWORD : DLsite アカウントのパスワード
 */
export async function loginToDlsite(page: Page): Promise<void> {
  const email = process.env.DLSITE_EMAIL
  const password = process.env.DLSITE_PASSWORD

  if (!email || !password) {
    throw new Error(
      'DLSITE_EMAIL または DLSITE_PASSWORD 環境変数が設定されていません。\n'
      + '実行前に以下のように設定してください:\n'
      + '  $env:DLSITE_EMAIL="your@email.com"\n'
      + '  $env:DLSITE_PASSWORD="yourpassword"',
    )
  }

  await page.goto(DLSITE_LOGIN_URL, { waitUntil: 'commit', timeout: NAVIGATION_TIMEOUT_MS })
  await waitBetweenSteps(page)
  await fillFirstAvailable(page, [
    '#form_id',
    'input[placeholder="ログインID"]',
    'input[name="login_id"]',
    'input[name="email"]',
    'input[type="email"]',
    'input[type="text"]',
    '[autocomplete="username"]',
  ], email)
  await fillFirstAvailable(page, [
    '#form_password',
    'input[placeholder="パスワード"]',
    'input[type="password"]',
    'input[name="password"]',
    '[autocomplete="current-password"]',
  ], password)
  await clickFirstAvailable(page, [
    'form button[type="submit"]',
    'button[type="submit"]',
    'input[type="submit"]',
    'button:has-text("ログイン")',
    'form button:has-text("ログイン")',
  ])

  await page.waitForURL((url) => {
    return (
      url.hostname !== 'login.dlsite.com'
      || url.pathname === '/guide/welcome'
    )
  }, {
    timeout: NAVIGATION_TIMEOUT_MS,
  })
  await waitBetweenSteps(page)
}

/**
 * DLsite の購入済み作品一覧ページへ遷移する。
 */
export async function navigateToDlsiteLibrary(page: Page): Promise<void> {
  await page.goto(DLSITE_LIBRARY_URL, { waitUntil: 'domcontentloaded', timeout: NAVIGATION_TIMEOUT_MS })
  await waitBetweenSteps(page)
  await page.waitForURL(url => url.hostname === 'play.dlsite.com' && url.pathname.includes('/library'), {
    timeout: NAVIGATION_TIMEOUT_MS,
  })
  await waitBetweenSteps(page)
}

/**
 * DLsite ライブラリの同期ボタンを押して、購入済み一覧の再取得を開始する。
 */
export async function triggerDlsiteLibrarySync(page: Page): Promise<void> {
  await clickFirstAvailable(page, [
    'button:has-text("ライブラリを同期")',
    'button:has-text("Synchronize Library")',
    '[role="button"]:has-text("ライブラリを同期")',
    '[role="button"]:has-text("Synchronize Library")',
    'a:has-text("ライブラリを同期")',
    'a:has-text("Synchronize Library")',
  ])
}
