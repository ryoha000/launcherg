import { describe, expect, it, vi } from 'vitest'
import { debug, extractStoreIdFromUrl, generateRequestId } from './utils'

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

  describe('extractStoreIdFromUrl', () => {
    it('rJコードを含むURLからstore_idを抽出する', () => {
      const url = 'https://example.com/RJ123456.jpg'
      expect(extractStoreIdFromUrl(url)).toBe('RJ123456')
    })

    it('vJコードを含むURLからstore_idを抽出する', () => {
      const url = 'https://example.com/path/VJ987654_thumb.png'
      expect(extractStoreIdFromUrl(url)).toBe('VJ987654')
    })

    it('bJコードを含むURLからstore_idを抽出する', () => {
      const url = 'https://example.com/BJ111222.webp'
      expect(extractStoreIdFromUrl(url)).toBe('BJ111222')
    })

    it('パスを含むURLからもstore_idを抽出する', () => {
      const url = 'https://play.dlsite.com/images/work/doujin/RJ300000/RJ299999_img_main.jpg'
      expect(extractStoreIdFromUrl(url)).toBe('RJ299999')
    })

    it('コードが含まれないURLの場合はnullを返す', () => {
      const url = 'https://example.com/image.jpg'
      expect(extractStoreIdFromUrl(url)).toBeNull()
    })

    it('空のURLの場合はnullを返す', () => {
      expect(extractStoreIdFromUrl('')).toBeNull()
    })
  })

  describe('debug', () => {
    it('debugModeがtrueの場合、メッセージを出力する', () => {
      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      debug(true, 'Test message', 'arg1', 'arg2')
      expect(consoleSpy).toHaveBeenCalledWith('[DLsite Extractor] Test message', 'arg1', 'arg2')
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
