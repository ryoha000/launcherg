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
    expect(games.length).toBeGreaterThan(0)

    const first = games[0] as DmmExtractedGame
    expect(first.store_id).toBeTruthy()
    expect(first.title).toBeTruthy()
  })
})
