import type { DlsiteExtractedGame } from '../src/types'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { JSDOM } from 'jsdom'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { extractAllGames, shouldExtract } from '../src/dom-extractor'

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
    const games = extractAllGames()

    const expectedGames: DlsiteExtractedGame[] = [{
      storeId: 'VJ01004076',
      category: 'pro',
      title: '【通常版】神様ちゅ～ず！ センセー女の子似合ってるよっ！',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01005000/VJ01004076_img_main_300x300.jpg',
    }, {
      storeId: 'VJ01000588',
      category: 'pro',
      title: 'DLsite限定セット特典 『ハルカナソラ キャラクターソング＆特典ドラマデジタル音源集』',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000588_img_main_300x300.jpg',
    }, {
      storeId: 'VJ01000576',
      category: 'pro',
      title: 'ハルカナソラ',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000576_img_main_300x300.jpg',
    }, {
      storeId: 'VJ01000575',
      category: 'pro',
      title: 'ヨスガノソラ',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000575_img_main_300x300.jpg',
    }, {
      storeId: 'RJ358346',
      category: 'maniax',
      title: 'ニセモノ聖女の邪教討伐',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ359000/RJ358346_img_main_300x300.jpg',
    }, {
      storeId: 'RJ01225565',
      category: 'maniax',
      title: '玩具戦記 メス×ガキ・リビルドー',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01226000/RJ01225565_img_main_300x300.jpg',
    }, {
      storeId: 'RJ01144692',
      category: 'maniax',
      title: 'ミラージュの離反',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01145000/RJ01144692_img_main_300x300.jpg',
    }, {
      storeId: 'VJ01004243',
      category: 'pro',
      title: '【通常版】メイドちゃんは迷途ちゅう',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01005000/VJ01004243_img_main_300x300.jpg',
    }, {
      storeId: 'RJ01380674',
      category: 'maniax',
      title: '満車率300% 3≒:Append.1 鉄板ギャル御乗車ぱっち',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01381000/RJ01380674_img_main_300x300.jpg',
    }, {
      storeId: 'RJ01221390',
      category: 'maniax',
      title: '黒の迷宮',
      thumbnailUrl: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01222000/RJ01221390_img_main_300x300.jpg',
    }]

    expect(games).toEqual(expectedGames)
  })
})
