import type { Page } from '@playwright/test'
import process from 'node:process'

const DMM_LOGIN_URL = 'https://accounts.dmm.co.jp/service/login/password/'
const DMM_MYLIBRARY_URL = 'https://dlsoft.dmm.co.jp/mylibrary/'
const DMM_AGE_CHECK_URL = 'https://www.dmm.co.jp/age_check/'

async function passDmmAgeCheck(page: Page): Promise<void> {
  const isAgeCheckPage = () => page.url().startsWith(DMM_AGE_CHECK_URL)

  if (!isAgeCheckPage()) {
    return
  }

  await page.getByRole('link', { name: 'はい' }).click()
  await page.waitForURL(url => !url.toString().startsWith(DMM_AGE_CHECK_URL), {
    timeout: 30_000,
  })
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

  await page.goto(DMM_LOGIN_URL, { waitUntil: 'networkidle' })

  // メールアドレス入力
  await page.locator('input[name="login_id"]').fill(email)
  // パスワード入力
  await page.locator('input[name="password"]').fill(password)
  // ログインボタンクリック
  await page.getByRole('button', { name: 'ログイン' }).click()

  // ログイン後のリダイレクトを待つ
  await page.waitForURL(url => !url.toString().includes('accounts.dmm.co.jp'), {
    timeout: 30_000,
  })
  await passDmmAgeCheck(page)
}

/**
 * DMM マイライブラリページへ遷移する。
 * コンテンツの読み込みが終わるまで待機する。
 */
export async function navigateToMyLibrary(page: Page): Promise<void> {
  await page.goto(DMM_MYLIBRARY_URL, { waitUntil: 'networkidle' })
  await passDmmAgeCheck(page)

  // マイライブラリのルート要素が表示されるまで待機
  await page.locator('#mylibrary').waitFor({ state: 'visible', timeout: 30_000 })
}
