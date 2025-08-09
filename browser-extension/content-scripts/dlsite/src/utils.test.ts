import { describe, expect, it } from 'vitest'
import { extractStoreIdFromUrl } from './utils'

describe('dLsite utils', () => {
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
})
