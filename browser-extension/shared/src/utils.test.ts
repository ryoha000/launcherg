import { describe, expect, it } from 'vitest'
import { generateRequestId } from './utils'

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
})
