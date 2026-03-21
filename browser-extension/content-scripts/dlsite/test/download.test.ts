import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it } from 'vitest'
import { findZipDownloadButton, parseLaunchergParam } from '../src/download'

describe('dlsite download helpers', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    // Reset URL (same-origin path only for JSDOM)
    window.history.replaceState({}, '', '/')
  })

  it('parseLaunchergParam: クエリが無ければ null', () => {
    const p = parseLaunchergParam()
    expect(p).toBeNull()
  })

  it('parseLaunchergParam: 正しい JSON を返す', () => {
    const payload = { type: 'download', value: { game: { storeId: 'RJ01234567' } } }
    const url = `/?launcherg=${encodeURIComponent(JSON.stringify(payload))}`
    window.history.pushState({}, '', url)

    const p = parseLaunchergParam()
    expect(p).not.toBeNull()
    expect(p?.type).toBe('download')
    expect(p?.value.game.storeId).toBe('RJ01234567')
  })

  it('findZipDownloadButton: a[href*=/api/v3/download] を返す', () => {
    const html = readFileSync(resolve(__dirname, 'data', 'dlsite_detail.html'), 'utf-8')
    const parser = new DOMParser()
    const doc = parser.parseFromString(html, 'text/html')
    const btn = findZipDownloadButton(doc) as HTMLAnchorElement | null
    expect(btn).not.toBeNull()
    expect(btn?.tagName).toBe('A')
    expect(btn?.getAttribute('href') || '').toContain('/api/v3/download')
  })
})
