import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { JSDOM } from 'jsdom'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { processGames } from '../src/data-processor'
import { extractAllGames, shouldExtract } from '../src/dom-extractor'
import { extractStoreIdFromUrl } from '../src/utils'

// __dirname の代替
const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// DLsite Play.htmlファイルのパス
const dlsitePlayHtmlPath = resolve(__dirname, './data/DLsite Play.html')

describe('dLsite Play.html E2Eテスト（JSDOMバックエンド）', () => {
  let dom: JSDOM
  let originalDocument: Document
  let originalWindow: Window & typeof globalThis

  beforeEach(() => {
    // 元のglobalオブジェクトを保存
    originalDocument = globalThis.document
    originalWindow = globalThis.window

    // HTMLファイルを読み込み
    const htmlContent = readFileSync(dlsitePlayHtmlPath, 'utf-8')

    // JSDOMインスタンスを作成（URLを指定することでlocationが自動設定される）
    dom = new JSDOM(htmlContent, {
      url: 'https://play.dlsite.com/library',
      pretendToBeVisual: true,
      resources: 'usable',
    })

    // グローバルオブジェクトを設定
    globalThis.document = dom.window.document
    globalThis.window = dom.window as any
    globalThis.HTMLElement = dom.window.HTMLElement
    globalThis.Element = dom.window.Element
    globalThis.NodeList = dom.window.NodeList
  })

  afterEach(() => {
    // グローバルオブジェクトを復元
    globalThis.document = originalDocument
    globalThis.window = originalWindow
    dom.window.close()
  })

  it('実際のDLsite Play.htmlファイルから正しくゲーム情報を抽出できること', () => {
    // 抽出対象ページかどうかを確認
    const rootElement = document.getElementById('root')
    const isTargetPage = shouldExtract('play.dlsite.com', rootElement)
    expect(isTargetPage).toBe(true)

    // ゲーム情報を抽出
    const games = extractAllGames(false) // debugMode = false

    console.log(`抽出されたゲーム数: ${games.length}`)

    // 基本的な検証
    expect(games.length).toBeGreaterThan(0)
    expect(games.length).toBeGreaterThanOrEqual(3)

    // 各ゲームデータの構造検証
    for (const game of games) {
      expect(game.store_id).toBeTruthy()
      expect(game.store_id).toMatch(/^(RJ|VJ|BJ)\d+$/)
      expect(game.title).toBeTruthy()
      expect(game.purchase_url).toContain('dlsite.com')
      expect(game.thumbnail_url).toContain('img.dlsite.jp')
      expect(game.additional_data.maker_name).toBeTruthy()
    }

    // 最初のゲームをログ出力
    if (games.length > 0) {
      console.log('最初のゲーム:', games[0])
    }
  })

  it('データ処理パイプライン全体が正しく動作すること', () => {
    // 生データを抽出
    const rawGames = extractAllGames(false)

    // データ処理を適用
    const processedGames = processGames(rawGames)

    expect(processedGames.length).toBe(rawGames.length)

    // 処理後のデータを検証
    for (const game of processedGames) {
      // URLが正規化されている
      expect(game.purchase_url).toMatch(/^https:\/\//)
      expect(game.thumbnail_url).toMatch(/^https:\/\//)

      // 追加データが設定されている
      expect(game.additional_data.store_name).toBe('DLsite')
      expect(game.additional_data.extraction_source).toBe('dlsite-extractor')
      expect(game.additional_data.extraction_timestamp).toBeTruthy()
      expect(game.additional_data.work_type).toMatch(/^(doujin|voice|book|unknown)$/)

      // 日付が正規化されている（存在する場合）
      if (game.purchase_date) {
        expect(game.purchase_date).toMatch(/^\d{4}-\d{2}-\d{2}$/)
      }
    }
  })

  it('hTMLにDLsiteの必要な要素が含まれていることを確認する', () => {
    const rootElement = document.getElementById('root')
    expect(rootElement).toBeTruthy()

    const thumbnails = document.querySelectorAll('._thumbnail_1kd4u_117')
    expect(thumbnails.length).toBeGreaterThan(0)

    const dataIndexElements = document.querySelectorAll('[data-index]')
    expect(dataIndexElements.length).toBeGreaterThan(0)

    const thumbnailSpans = document.querySelectorAll('._thumbnail_1kd4u_117 span')
    expect(thumbnailSpans.length).toBeGreaterThan(0)

    console.log('DOM要素チェック結果:', {
      hasRoot: !!rootElement,
      thumbnailCount: thumbnails.length,
      dataIndexCount: dataIndexElements.length,
      thumbnailSpanCount: thumbnailSpans.length,
    })
  })

  it('store_id抽出の正確性を個別にテストする', () => {
    const thumbnailElements = document.querySelectorAll('._thumbnail_1kd4u_117 span')
    const results: Array<{
      index: number
      url: string
      storeId: string | null
      valid: boolean
    }> = []

    thumbnailElements.forEach((element, index) => {
      const htmlElement = element as HTMLElement
      const bgImage = htmlElement.style.backgroundImage
      const match = bgImage.match(/url\("?(.+?)"?\)/)

      if (match) {
        const url = match[1]
        const storeId = extractStoreIdFromUrl(url)
        results.push({
          index,
          url,
          storeId,
          valid: storeId ? /^(?:RJ|VJ|BJ)\d+$/.test(storeId) : false,
        })
      }
    })

    console.log(`Store ID抽出テスト: ${results.length}個のサムネイルを処理`)

    // 検証
    expect(results.length).toBeGreaterThan(0)

    const validExtractions = results.filter(result => result.valid)
    expect(validExtractions.length).toBeGreaterThan(0)

    // すべての有効なstore_idが正しい形式であることを確認
    for (const result of validExtractions) {
      expect(result.storeId).toMatch(/^(RJ|VJ|BJ)\d+$/)
      expect(result.url).toContain('img.dlsite.jp')
    }

    // 最初の有効な抽出結果をログ出力
    if (validExtractions.length > 0) {
      console.log('最初の有効なstore_id抽出:', validExtractions[0])
    }
  })

  it('重複排除が正しく機能すること', () => {
    // テスト用に同じゲームが複数回出現するHTMLを作成
    const testHtml = `
      <div id="root">
        <div data-index="0">
          <div class="_thumbnail_1kd4u_117">
            <span style="background-image: url('https://img.dlsite.jp/RJ123456.jpg')"></span>
          </div>
          <div class="_workName_1kd4u_192"><span>Test Game 1</span></div>
          <div class="_makerName_1kd4u_196"><span>Test Maker</span></div>
        </div>
        <div data-index="1">
          <div class="_thumbnail_1kd4u_117">
            <span style="background-image: url('https://img.dlsite.jp/RJ123456.jpg')"></span>
          </div>
          <div class="_workName_1kd4u_192"><span>Test Game 1 Duplicate</span></div>
          <div class="_makerName_1kd4u_196"><span>Test Maker</span></div>
        </div>
        <div data-index="2">
          <div class="_thumbnail_1kd4u_117">
            <span style="background-image: url('https://img.dlsite.jp/VJ987654.jpg')"></span>
          </div>
          <div class="_workName_1kd4u_192"><span>Test Game 2</span></div>
          <div class="_makerName_1kd4u_196"><span>Test Maker 2</span></div>
        </div>
      </div>
    `

    // 新しいJSDOMインスタンスでテスト
    const testDom = new JSDOM(testHtml)
    globalThis.document = testDom.window.document

    const games = extractAllGames(false)

    // RJ123456が重複していても、1つしか抽出されないことを確認
    expect(games.length).toBe(2)

    const storeIds = games.map(g => g.store_id)
    expect(storeIds).toContain('RJ123456')
    expect(storeIds).toContain('VJ987654')

    // 重複がないことを確認
    const uniqueStoreIds = new Set(storeIds)
    expect(uniqueStoreIds.size).toBe(games.length)

    testDom.window.close()
  })
})
