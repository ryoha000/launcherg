import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { collectDownloadLinks } from '../src/download-modal'

function loadFixture(name: string): Document {
  const html = readFileSync(resolve(__dirname, 'data', name), 'utf-8')
  const parser = new DOMParser()
  return parser.parseFromString(html, 'text/html')
}

describe('ダウンロードリンク抽出', () => {
  it('単一ダウンロード: 「ダウンロード」ボタンを1つ抽出する', () => {
    const doc = loadFixture('dmm_open_modal_only_single_download.html')
    const links = collectDownloadLinks(doc)
    expect(links.length).toBe(1)
    expect(links[0].textContent?.includes('ダウンロード')).toBe(true)
    expect(links[0].getAttribute('href') || '').toContain('/mylibrary/proxy')
  })

  it('結合+分割ダウンロード: 全てのリンクを抽出する', () => {
    const doc = loadFixture('dmm_open_modal_concat_download.html')
    const links = collectDownloadLinks(doc)
    // 1件の結合 + 2件の分割 = 3
    expect(links.length).toBe(3)
    const hrefs = links.map(a => a.getAttribute('href') || '')
    expect(hrefs.filter(h => h.includes('combining=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=2')).length).toBe(1)
  })
})
