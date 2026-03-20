import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  buildDmmPayloadKey,
  convertDmmLibraryItem,
  extractDmmGamesFromApiResponse,
  isDmmLibraryApiUrl,
  splitDmmApiGames,
} from '../src/api'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

describe('dmm api parser', () => {
  it('dMM library API URL を判定できる', () => {
    expect(isDmmLibraryApiUrl('https://dlsoft.dmm.co.jp/ajax/v1/library?page=1')).toBe(true)
    expect(isDmmLibraryApiUrl('https://dlsoft.dmm.co.jp/mylibrary/')).toBe(false)
  })

  it('sample_dmm.json から single と set を正しく変換できる', () => {
    const jsonPath = resolve(__dirname, '../../../tests/unit/data/sample_dmm.json')
    const payload = JSON.parse(readFileSync(jsonPath, 'utf-8'))
    const games = extractDmmGamesFromApiResponse(payload)
    const { normalGames, packGames } = splitDmmApiGames(games)

    expect(normalGames).toEqual([{
      storeId: 'ncpy_0007',
      category: 'digital',
      subcategory: 'pcgame',
      title: 'Monkeys!¡',
      imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/ncpy_0007/ncpy_0007ps.jpg',
    }])
    expect(packGames).toHaveLength(1)
    expect(packGames[0]).toMatchObject({
      storeId: 'purple_0028pack',
      isPack: true,
      category: 'digital',
      subcategory: 'pcgame',
      title: '天つ籠ノ鳥BOX',
      imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/purple_0028pack/purple_0028packps.jpg',
    })
  })

  it('payload key は同一 payload で安定する', () => {
    const message = {
      source: 'launcherg' as const,
      type: 'launcherg:dmm-library-response' as const,
      pageUrl: 'https://dlsoft.dmm.co.jp/mylibrary/',
      requestUrl: 'https://dlsoft.dmm.co.jp/ajax/v1/library?page=1',
      payload: {
        error: null,
        body: {
          library: [
            { productId: 'a', libraryProductType: 'single' },
            { productId: 'b', libraryProductType: 'set' },
          ],
        },
      },
    }

    expect(buildDmmPayloadKey(message)).toBe(buildDmmPayloadKey({ ...message }))
  })

  it('必要項目が欠けた item は無視する', () => {
    expect(convertDmmLibraryItem({ title: 'title only' })).toBeNull()
  })
})
