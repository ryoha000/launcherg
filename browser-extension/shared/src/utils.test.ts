import { describe, expect, it } from 'vitest'
import { generateRequestId, normalizeTitle } from './utils'

describe('utils', () => {
  describe('generateRequestId', () => {
    it('一意のIDを生成する', () => {
      const id1 = generateRequestId()
      const id2 = generateRequestId()
      expect(id1).toBeTruthy()
      expect(id2).toBeTruthy()
      expect(id1).not.toBe(id2)
    })

    it('正しい形式のIDを生成する', () => {
      const id = generateRequestId()
      // タイムスタンプとランダム文字列を含む
      expect(id).toMatch(/^[0-9a-z]+$/)
    })
  })

  // debug() は logger に置換したためテスト対象外
  describe('normalizeTitle', () => {
    it('角括弧【…】を除去する', () => {
      expect(normalizeTitle('タイトル【受賞】')).toBe('タイトル')
      expect(normalizeTitle('【タグ】タイトル')).toBe('タイトル')
    })

    it('全角/半角の括弧を除去する', () => {
      expect(normalizeTitle('ゲーム（追加情報）タイトル')).toBe('ゲームタイトル')
      expect(normalizeTitle('ゲーム(追加情報)タイトル')).toBe('ゲームタイトル')
      expect(normalizeTitle('（前情報）ゲームタイトル')).toBe('ゲームタイトル')
      expect(normalizeTitle('(前情報)ゲームタイトル')).toBe('ゲームタイトル')
    })

    it('角括弧[...]の注釈を除去する', () => {
      expect(normalizeTitle('[サークル名] ゲームタイトル')).toBe('ゲームタイトル')
      expect(normalizeTitle('ゲーム[情報]タイトル')).toBe('ゲームタイトル')
    })

    it('連続空白を1つにして前後空白を除去する', () => {
      expect(normalizeTitle('  ゲーム   タイトル  ')).toBe('ゲーム タイトル')
    })

    it('複合ケースに対応する', () => {
      expect(normalizeTitle('[サークル] ゲーム（バージョン）(English)【受賞】')).toBe('ゲーム')
    })
  })
})
