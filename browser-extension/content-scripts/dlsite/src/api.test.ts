import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import {
  buildDlsitePayloadKey,
  convertDlsiteWorkItem,
  extractDlsiteGamesFromApiResponse,
  isDlsiteWorksApiUrl,
} from './api'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

describe('dlsite api parser', () => {
  it('dLsite works API URL を判定できる', () => {
    expect(isDlsiteWorksApiUrl('https://play.dlsite.com/api/v3/content/works')).toBe(true)
    expect(isDlsiteWorksApiUrl('https://play.dlsite.com/library')).toBe(false)
  })

  it('sample_dlsite.json からゲーム一覧を正しく変換できる', () => {
    const jsonPath = resolve(__dirname, '../../../tests/unit/data/sample_dlsite.json')
    const payload = JSON.parse(readFileSync(jsonPath, 'utf-8'))
    const games = extractDlsiteGamesFromApiResponse(payload)

    expect(games).toEqual([{
      storeId: 'RJ01007737',
      category: 'maniax',
      title: 'クロア×スクランブル',
      imageUrl: 'https://img.dlsite.jp/modpub/images2/work/doujin/RJ01008000/RJ01007737_img_main.jpg',
    }])
  })

  it('payload key は同一 payload で安定する', () => {
    const message = {
      source: 'launcherg' as const,
      type: 'launcherg:dlsite-works-response' as const,
      pageUrl: 'https://play.dlsite.com/library',
      requestUrl: 'https://play.dlsite.com/api/v3/content/works',
      payload: {
        works: [
          { workno: 'RJ1', site_id: 'maniax' },
          { workno: 'VJ2', site_id: 'pro' },
        ],
      },
    }

    expect(buildDlsitePayloadKey(message)).toBe(buildDlsitePayloadKey({ ...message }))
  })

  it('必要項目が欠けた作品は無視する', () => {
    expect(convertDlsiteWorkItem({ workno: 'RJ1' })).toBeNull()
  })
})
