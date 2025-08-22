import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { JSDOM } from 'jsdom'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { findDetailItemIdForStoreId } from '../src/pack-helpers'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const htmlPath = resolve(__dirname, './data/dmm.html')

describe('findDetailItemIdForStoreId', () => {
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

  it('storeIdがnavel_0004のとき、itemIdがFQJXWAtTHVxZE1RUbAZWB1U_である', () => {
    const itemId = findDetailItemIdForStoreId('navel_0004')
    expect(itemId).toBe('FQJXWAtTHVxZE1RUbAZWB1U_')
  })
})
