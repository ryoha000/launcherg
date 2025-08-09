import type { DmmExtractedGame } from '../src/types'
import { describe, expect, it } from 'vitest'
import { normalizeDmmDate, normalizeUrl, processDmmGame, processGames } from '../src/data-processor'

describe('dmm data-processor', () => {
  it('normalizeDmmDate', () => {
    expect(normalizeDmmDate('2024年1月2日')).toBe('2024-01-02')
    expect(normalizeDmmDate('2024/1/2')).toBe('2024-01-02')
    expect(normalizeDmmDate('2024-1-2')).toBe('2024-01-02')
    expect(normalizeDmmDate('bad')).toBe('bad')
  })

  it('normalizeUrl', () => {
    expect(normalizeUrl('//img.example/a.jpg', 'thumbnail')).toBe('https://img.example/a.jpg')
    expect(normalizeUrl('/path', 'purchase')).toBe('https://dlsoft.dmm.co.jp/path')
    expect(normalizeUrl('https://x', 'purchase')).toBe('https://x')
  })

  it('processDmmGame', () => {
    const g: DmmExtractedGame = {
      store_id: 'abc_1',
      title: 't',
      purchase_url: '/mylibrary/?cid=abc_1',
      purchase_date: '2024年1月2日',
      thumbnail_url: '//img/a.jpg',
      additional_data: {},
    }
    const r = processDmmGame(g)
    expect(r.purchase_url).toBe('https://dlsoft.dmm.co.jp/mylibrary/?cid=abc_1')
    expect(r.purchase_date).toBe('2024-01-02')
    expect(r.thumbnail_url).toBe('https://img/a.jpg')
    expect(r.additional_data.store_name).toBe('DMM')
  })

  it('processGames', () => {
    const arr: DmmExtractedGame[] = [
      { store_id: 'a', title: 't', purchase_url: '/x', additional_data: {} },
    ]
    const res = processGames(arr)
    expect(res.length).toBe(1)
  })
})
