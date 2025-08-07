import type { ExtractedGameData } from '@launcherg/shared'
import { describe, expect, it } from 'vitest'
import {
  cleanDLsiteTitle,
  determineWorkType,
  normalizeDLsiteDate,
  normalizeStoreId,
  normalizeUrl,
  processDLsiteGame,
  processGames,
} from './data-processor'

describe('data-processor', () => {
  describe('normalizeDLsiteDate', () => {
    it('日本語形式の日付を正規化する', () => {
      expect(normalizeDLsiteDate('2024年1月15日')).toBe('2024-01-15')
      expect(normalizeDLsiteDate('2023年12月31日')).toBe('2023-12-31')
    })

    it('スラッシュ形式の日付を正規化する', () => {
      expect(normalizeDLsiteDate('2024/01/15')).toBe('2024-01-15')
      expect(normalizeDLsiteDate('2023/12/31')).toBe('2023-12-31')
    })

    it('ハイフン形式の日付を正規化する', () => {
      expect(normalizeDLsiteDate('2024-01-15')).toBe('2024-01-15')
      expect(normalizeDLsiteDate('2023-12-31')).toBe('2023-12-31')
    })

    it('空白を含む日付を正規化する', () => {
      expect(normalizeDLsiteDate('2024年 1月 15日')).toBe('2024-01-15')
    })

    it('不正な日付の場合は元の文字列を返す', () => {
      expect(normalizeDLsiteDate('invalid date')).toBe('invalid date')
      expect(normalizeDLsiteDate('')).toBe('')
    })
  })

  describe('cleanDLsiteTitle', () => {
    it('[サークル名]を除去する', () => {
      expect(cleanDLsiteTitle('[サークル名] ゲームタイトル')).toBe('ゲームタイトル')
      expect(cleanDLsiteTitle('ゲーム[情報]タイトル')).toBe('ゲームタイトル')
    })

    it('全角括弧の内容を除去する', () => {
      expect(cleanDLsiteTitle('ゲーム（追加情報）タイトル')).toBe('ゲームタイトル')
      expect(cleanDLsiteTitle('（前情報）ゲームタイトル')).toBe('ゲームタイトル')
    })

    it('半角括弧の内容を除去する', () => {
      expect(cleanDLsiteTitle('ゲーム(追加情報)タイトル')).toBe('ゲームタイトル')
      expect(cleanDLsiteTitle('(前情報)ゲームタイトル')).toBe('ゲームタイトル')
    })

    it('連続する空白を単一の空白に正規化する', () => {
      expect(cleanDLsiteTitle('ゲーム   タイトル')).toBe('ゲーム タイトル')
      expect(cleanDLsiteTitle('  ゲーム  タイトル  ')).toBe('ゲーム タイトル')
    })

    it('複数の括弧を処理する', () => {
      expect(cleanDLsiteTitle('[サークル] ゲーム（バージョン）(English)')).toBe('ゲーム')
    })
  })

  describe('normalizeUrl', () => {
    it('プロトコルなしのURLにhttpsを追加する', () => {
      expect(normalizeUrl('//example.com/image.jpg', 'thumbnail')).toBe('https://example.com/image.jpg')
      expect(normalizeUrl('/path/to/image.jpg', 'purchase')).toBe('https://play.dlsite.com/path/to/image.jpg')
    })

    it('既にhttpプロトコルがある場合はそのまま返す', () => {
      expect(normalizeUrl('http://example.com/page', 'purchase')).toBe('http://example.com/page')
      expect(normalizeUrl('https://example.com/page', 'thumbnail')).toBe('https://example.com/page')
    })

    it('空のURLの場合はそのまま返す', () => {
      expect(normalizeUrl('', 'purchase')).toBe('')
    })
  })

  describe('normalizeStoreId', () => {
    it('uRLから作品コードを抽出する', () => {
      expect(normalizeStoreId('123456', 'https://example.com/RJ123456')).toBe('RJ123456')
      expect(normalizeStoreId('wrong', 'https://example.com/VJ987654')).toBe('VJ987654')
      expect(normalizeStoreId('wrong', 'https://example.com/BJ111222')).toBe('BJ111222')
    })

    it('正しい形式のstore_idはそのまま返す', () => {
      expect(normalizeStoreId('RJ123456', 'https://example.com')).toBe('RJ123456')
      expect(normalizeStoreId('VJ987654', 'https://example.com')).toBe('VJ987654')
      expect(normalizeStoreId('BJ111222', 'https://example.com')).toBe('BJ111222')
    })

    it('数字のみの場合はRJを付加する', () => {
      expect(normalizeStoreId('123456', 'https://example.com')).toBe('RJ123456')
      expect(normalizeStoreId('987654', 'https://example.com')).toBe('RJ987654')
    })

    it('空のstore_idの場合はそのまま返す', () => {
      expect(normalizeStoreId('', 'https://example.com')).toBe('')
    })

    it('不正な形式の場合はそのまま返す', () => {
      expect(normalizeStoreId('invalid', 'https://example.com')).toBe('invalid')
    })
  })

  describe('determineWorkType', () => {
    it('rJコードの場合はdoujinを返す', () => {
      expect(determineWorkType('RJ123456')).toBe('doujin')
    })

    it('vJコードの場合はvoiceを返す', () => {
      expect(determineWorkType('VJ123456')).toBe('voice')
    })

    it('bJコードの場合はbookを返す', () => {
      expect(determineWorkType('BJ123456')).toBe('book')
    })

    it('その他の場合はunknownを返す', () => {
      expect(determineWorkType('XX123456')).toBe('unknown')
      expect(determineWorkType('123456')).toBe('unknown')
      expect(determineWorkType('')).toBe('unknown')
    })
  })

  describe('processDLsiteGame', () => {
    const baseGame: ExtractedGameData = {
      store_id: '123456',
      title: '[サークル名] ゲーム（バージョン1.0）',
      purchase_url: '/work/RJ123456',
      purchase_date: '2024年1月15日',
      thumbnail_url: '//example.com/thumb.jpg',
      additional_data: {
        maker_name: 'Test Maker',
      },
    }

    it('ゲームデータを正しく処理する', () => {
      const result = processDLsiteGame(baseGame)

      expect(result.store_id).toBe('RJ123456')
      expect(result.title).toBe('ゲーム')
      expect(result.purchase_url).toBe('https://play.dlsite.com/work/RJ123456')
      expect(result.purchase_date).toBe('2024-01-15')
      expect(result.thumbnail_url).toBe('https://example.com/thumb.jpg')
      expect(result.additional_data.store_name).toBe('DLsite')
      expect(result.additional_data.extraction_source).toBe('dlsite-extractor')
      expect(result.additional_data.work_type).toBe('doujin')
      expect(result.additional_data.maker_name).toBe('Test Maker')
    })

    it('元のオブジェクトを変更しない（不変性）', () => {
      const originalData = { ...baseGame.additional_data }
      processDLsiteGame(baseGame)

      expect(baseGame.additional_data).toEqual(originalData)
      expect(baseGame.store_id).toBe('123456')
    })

    it('vJコードの作品を正しく処理する', () => {
      const voiceGame: ExtractedGameData = {
        ...baseGame,
        store_id: 'VJ987654',
      }

      const result = processDLsiteGame(voiceGame)
      expect(result.additional_data.work_type).toBe('voice')
    })

    it('bJコードの作品を正しく処理する', () => {
      const bookGame: ExtractedGameData = {
        ...baseGame,
        store_id: 'BJ111222',
      }

      const result = processDLsiteGame(bookGame)
      expect(result.additional_data.work_type).toBe('book')
    })
  })

  describe('processGames', () => {
    it('複数のゲームを処理する', () => {
      const games: ExtractedGameData[] = [
        {
          store_id: '123456',
          title: '[サークル1] ゲーム1',
          purchase_url: '/work/RJ123456',
          additional_data: {},
        },
        {
          store_id: 'VJ987654',
          title: '[サークル2] ゲーム2',
          purchase_url: '/work/VJ987654',
          additional_data: {},
        },
      ]

      const results = processGames(games)

      expect(results).toHaveLength(2)
      expect(results[0].store_id).toBe('RJ123456')
      expect(results[0].title).toBe('ゲーム1')
      expect(results[0].additional_data.work_type).toBe('doujin')
      expect(results[1].store_id).toBe('VJ987654')
      expect(results[1].title).toBe('ゲーム2')
      expect(results[1].additional_data.work_type).toBe('voice')
    })

    it('空の配列を処理する', () => {
      const results = processGames([])
      expect(results).toEqual([])
    })
  })
})
