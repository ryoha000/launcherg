import type { ExtractedGameData } from '@launcherg/shared'
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

    const expectedGames: ExtractedGameData[] = [{
      store_id: 'VJ01004076',
      title: '【通常版】神様ちゅ～ず！ センセー女の子似合ってるよっ！',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/VJ01004076.html',
      purchase_date: '2025年6月9日',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01005000/VJ01004076_img_main_300x300.jpg',
      additional_data: { maker_name: 'くまのみそふと' },
    }, {
      store_id: 'VJ01000588',
      title: 'DLsite限定セット特典 『ハルカナソラ キャラクターソング＆特典ドラマデジタル音源集』',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/VJ01000588.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000588_img_main_300x300.jpg',
      additional_data: { maker_name: 'Sphere' },
    }, {
      store_id: 'VJ01000576',
      title: 'ハルカナソラ',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/VJ01000576.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000576_img_main_300x300.jpg',
      additional_data: { maker_name: 'Sphere' },
    }, {
      store_id: 'VJ01000575',
      title: 'ヨスガノソラ',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/VJ01000575.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01001000/VJ01000575_img_main_300x300.jpg',
      additional_data: { maker_name: 'Sphere' },
    }, {
      store_id: 'RJ358346',
      title: 'ニセモノ聖女の邪教討伐',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ358346.html',
      purchase_date: '2025年6月1日',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ359000/RJ358346_img_main_300x300.jpg',
      additional_data: { maker_name: 'WhitePeach' },
    }, {
      store_id: 'RJ01225565',
      title: '玩具戦記 メス×ガキ・リビルドー',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ01225565.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01226000/RJ01225565_img_main_300x300.jpg',
      additional_data: { maker_name: '未亜見あみ' },
    }, {
      store_id: 'RJ01144692',
      title: 'ミラージュの離反',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ01144692.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01145000/RJ01144692_img_main_300x300.jpg',
      additional_data: { maker_name: 'ちまラボ' },
    }, {
      store_id: 'VJ01004243',
      title: '【通常版】メイドちゃんは迷途ちゅう',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/VJ01004243.html',
      purchase_date: '2025年5月31日',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/professional/VJ01005000/VJ01004243_img_main_300x300.jpg',
      additional_data: { maker_name: 'Clover GAME' },
    }, {
      store_id: 'RJ01380674',
      title: '満車率300% 3≒:Append.1 鉄板ギャル御乗車ぱっち',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ01380674.html',
      purchase_date: '',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01381000/RJ01380674_img_main_300x300.jpg',
      additional_data: { maker_name: 'ベルゼブブ' },
    }, {
      store_id: 'RJ01221390',
      title: '黒の迷宮',
      purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ01221390.html',
      purchase_date: '2025年5月29日',
      thumbnail_url: 'https://img.dlsite.jp/resize/images2/work/doujin/RJ01222000/RJ01221390_img_main_300x300.jpg',
      additional_data: { maker_name: '百舌鳥畑' },
    }]

    expect(games).toEqual(expectedGames)
  })
})
