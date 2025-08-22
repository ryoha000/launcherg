import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { parsePackModal } from '../src/pack-parser'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const htmlPath = resolve(__dirname, './data/dmm_open_modal.html')

describe('parsePackModal', () => {
  it('hTMLフィクスチャからパック内ゲームを抽出できる', () => {
    const html = readFileSync(htmlPath, 'utf-8')
    const games = parsePackModal(html)
    expect(games).toHaveLength(3)
    expect(games.map(g => g.storeId)).toEqual(['views_0528', 'views_0571', 'purple_0029'])

    const views0528 = games.find(g => g.storeId === 'views_0528')!
    expect(views0528).toMatchObject({
      category: 'digital',
      subcategory: 'pcgame',
      storeId: 'views_0528',
      title: 'アマツツミ',
    })
  })
})
