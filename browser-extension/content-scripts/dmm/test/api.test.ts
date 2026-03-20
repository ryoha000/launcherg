import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  buildDmmPayloadKey,
  convertDmmLibraryItem,
  extractDmmGamesFromSetDetailResponse,
  extractDmmGamesFromApiResponse,
  extractDownloadUrlsFromSetDetail,
  extractDownloadUrlsFromSingleDetail,
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

  it('sample_dmm_pack.json から childProducts を正しく変換できる', () => {
    const jsonPath = resolve(__dirname, '../../../tests/unit/data/sample_dmm_pack.json')
    const payload = JSON.parse(readFileSync(jsonPath, 'utf-8'))
    const games = extractDmmGamesFromSetDetailResponse(payload)

    expect(games).toEqual([
      {
        storeId: 'views_0528',
        category: 'digital',
        subcategory: 'pcgame',
        title: 'アマツツミ',
        imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/views_0528/views_0528ps.jpg',
      },
      {
        storeId: 'views_0571',
        category: 'digital',
        subcategory: 'pcgame',
        title: 'アオイトリ',
        imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/views_0571/views_0571ps.jpg',
      },
      {
        storeId: 'purple_0029',
        category: 'digital',
        subcategory: 'pcgame',
        title: '天つ籠ノ鳥BOX ハイレゾ音源サウンドトラック',
        imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/purple_0029/purple_0029ps.jpg',
      },
    ])
  })

  it('download detail API から URL を正しい順で抽出できる', () => {
    const notPackJsonPath = resolve(__dirname, '../../../tests/unit/data/sample_dmm_not_pack.json')
    const packJsonPath = resolve(__dirname, '../../../tests/unit/data/sample_dmm_pack.json')
    const notPackPayload = JSON.parse(readFileSync(notPackJsonPath, 'utf-8'))
    const packPayload = JSON.parse(readFileSync(packJsonPath, 'utf-8'))

    expect(extractDownloadUrlsFromSingleDetail(notPackPayload)).toEqual([
      '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.bat&productId=hobc_0157&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.000&productId=hobc_0157&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.001&productId=hobc_0157&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.002&productId=hobc_0157&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fhobc_0157%2Fhobc_0157.003&productId=hobc_0157&floor=Apcgame',
    ])

    expect(extractDownloadUrlsFromSetDetail(packPayload, 'views_0571')).toEqual([
      '/download/?filePath=%2Fbb%2Fpcgame%2Fviews_0571%2Fviews_0571.part1.exe&productId=views_0571&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fviews_0571%2Fviews_0571.part2.rar&productId=views_0571&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fviews_0571%2Fviews_0571.part3.rar&productId=views_0571&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fviews_0571%2Fviews_0571.part4.rar&productId=views_0571&floor=Apcgame',
      '/download/?filePath=%2Fbb%2Fpcgame%2Fviews_0571%2Fviews_0571.part5.rar&productId=views_0571&floor=Apcgame',
    ])
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
