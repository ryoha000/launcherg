import type { DmmExtractedGame } from '../src/types'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { JSDOM } from 'jsdom'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { extractAllGames, shouldExtract } from '../src/dom-extractor'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const htmlPath = resolve(__dirname, './data/dmm.html')

describe('dMM mylibrary e2e', () => {
  let dom: JSDOM
  let originalDocument: Document
  let originalWindow: Window & typeof globalThis

  beforeEach(() => {
    originalDocument = globalThis.document
    originalWindow = globalThis.window
    const html = readFileSync(htmlPath, 'utf-8')
    dom = new JSDOM(html, { url: 'https://dlsoft.dmm.co.jp/mylibrary/', pretendToBeVisual: true })
    globalThis.document = dom.window.document
    globalThis.window = dom.window as any
    globalThis.HTMLElement = dom.window.HTMLElement
    globalThis.Element = dom.window.Element
    globalThis.NodeList = dom.window.NodeList
  })

  afterEach(() => {
    globalThis.document = originalDocument
    globalThis.window = originalWindow
    dom.window.close()
  })

  it('フィクスチャからゲーム情報を抽出できる', () => {
    const root = document.getElementById('mylibrary')
    const ok = shouldExtract('dlsoft.dmm.co.jp', root)
    expect(ok).toBe(true)

    const games = extractAllGames()
    // ノイズのない商品カードに一致するよう、先頭から不要なバナー画像等を除いた後で比較
    const filtered = games.filter(g => /^(?:vsat_|feng_|nightingale_|gm_|has_|next_|mwnds_|akbs_)/.test(g.store_id))
    expect(filtered.length).toBeGreaterThanOrEqual(10)

    const expectedFirst10: DmmExtractedGame[] = [
      {
        store_id: 'vsat_0158',
        title: 'はつゆきさくら',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=vsat_0158',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/vsat_0158/vsat_0158ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'feng_0004',
        title: '妹のセイイキ【萌えゲーアワード2015 エロス系作品賞PINK 受賞】',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=feng_0004',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/feng_0004/feng_0004ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'nightingale_0001',
        title: '紙の上の魔法使い【萌えゲーアワード2014 ニューブランド賞受賞】',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=nightingale_0001',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/nightingale_0001/nightingale_0001ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'gm_whirlvc4',
        title: '【音楽】Whirlpool Original Vocal Collection4',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=gm_whirlvc4',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/gm_whirlvc4/gm_whirlvc4ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'has_0090',
        title: '恋愛×ロワイアル 乃々香＆蓮菜＆由奈 ミニアフターストーリー',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=has_0090',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/has_0090/has_0090ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'has_0123',
        title: '放課後シンデレラ2 ミニファンディスク 〜君と踊る初めてのハッピーハロウィン〜',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=has_0123',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/has_0123/has_0123ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'has_0145',
        title: 'コイバナ恋愛 ミニファンディスク アフターフェスティバル',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=has_0145',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/has_0145/has_0145ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'next_0442',
        title: '初恋マスターアップ',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=next_0442',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/next_0442/next_0442ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'mwnds_0015',
        title: 'ハミダシクリエイティブ Re：Re：call',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=mwnds_0015',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/mwnds_0015/mwnds_0015ps.jpg',
        additional_data: {},
      },
      {
        store_id: 'mwnds_0016',
        title: '【音楽】ハミダシクリエイティブ Re：Re：call エンディングボーカル（ハイレゾバージョン）',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=mwnds_0016',
        purchase_date: '',
        thumbnail_url: 'https://pics.dmm.co.jp/digital/pcgame/mwnds_0016/mwnds_0016ps.jpg',
        additional_data: {},
      },
    ]

    expect(filtered.slice(0, 10)).toEqual(expectedFirst10)
  })
})
