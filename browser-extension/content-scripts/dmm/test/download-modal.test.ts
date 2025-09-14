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
    const links = collectDownloadLinks(doc, 'hobe_0340')
    expect(links.length).toBe(1)
    expect(links[0].textContent?.includes('ダウンロード')).toBe(true)
    expect(links[0].getAttribute('href') || '').toContain('/mylibrary/proxy')
  })

  it('結合+分割ダウンロード: 全てのリンクを抽出する', () => {
    const doc = loadFixture('dmm_open_modal_concat_download.html')
    const links = collectDownloadLinks(doc, 'purple_0007')
    // 1件の結合 + 2件の分割 = 3
    expect(links.length).toBe(3)
    const hrefs = links.map(a => a.getAttribute('href') || '')
    expect(hrefs.filter(h => h.includes('combining=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=2')).length).toBe(1)
  })

  it('パック: 詳細未選択でもstoreId一致のリンクを抽出する', () => {
    const doc = loadFixture('dmm_open_modal_pack.html')
    const links = collectDownloadLinks(doc, 'next_0287')
    expect(links.length).toBe(4)
  })

  it('パック: 子A(next_0287)のリンクのみ抽出（結合1+分割3=4）', () => {
    const doc = loadFixture('dmm_open_modal_pack.html')
    const links = collectDownloadLinks(doc, 'next_0287')
    expect(links.length).toBe(4)
    const hrefs = links.map(a => a.getAttribute('href') || '')
    expect(hrefs.every(h => h.includes('pid=next_0287'))).toBe(true)
    expect(hrefs.filter(h => h.includes('combining=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=2')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=3')).length).toBe(1)
  })

  it('パック: 子B(next_0338)のリンクのみ抽出（単品1）', () => {
    const doc = loadFixture('dmm_open_modal_pack.html')
    ;(doc.getElementById('detail_JBFTXgdbBE5WAElMbAZVBFkZBw8GAVwPV1ED') as HTMLElement).style.display = ''
    const links = collectDownloadLinks(doc, 'next_0338')
    expect(links.length).toBe(1)
    expect(links[0].textContent?.includes('ダウンロード')).toBe(true)
    expect((links[0].getAttribute('href') || '').includes('pid=next_0338')).toBe(true)
  })

  it('パック: 子C(next_0370)のリンクのみ抽出（結合1+分割5=6）', () => {
    const doc = loadFixture('dmm_open_modal_pack.html')
    ;(doc.getElementById('detail_JBFTXgdbBE5WAElMbAZVAFEZBw8GAVwPV1ED') as HTMLElement).style.display = ''
    const links = collectDownloadLinks(doc, 'next_0370')
    expect(links.length).toBe(6)
    const hrefs = links.map(a => a.getAttribute('href') || '')
    expect(hrefs.every(h => h.includes('pid=next_0370'))).toBe(true)
    expect(hrefs.filter(h => h.includes('combining=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=1')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=2')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=3')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=4')).length).toBe(1)
    expect(hrefs.filter(h => h.includes('num=5')).length).toBe(1)
  })
})
