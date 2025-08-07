import { describe, expect, it, vi } from 'vitest'
import { debug, generateRequestId } from './utils'

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

  describe('debug', () => {
    it('debugModeがtrueの場合、メッセージを出力する', () => {
      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debug(true, 'Test message', 'arg1', 'arg2')
      expect(consoleSpy).toHaveBeenCalledWith('Test message', 'arg1', 'arg2')
      consoleSpy.mockRestore()
    })

    it('debugModeがfalseの場合、メッセージを出力しない', () => {
      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debug(false, 'Test message', 'arg1', 'arg2')
      expect(consoleSpy).not.toHaveBeenCalled()
      consoleSpy.mockRestore()
    })
  })
})
