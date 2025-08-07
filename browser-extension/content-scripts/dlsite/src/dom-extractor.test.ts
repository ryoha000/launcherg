import { beforeEach, describe, expect, it, vi } from 'vitest'
import { extractGameDataFromContainer, shouldExtract } from './dom-extractor'

describe('dom-extractor', () => {
  describe('shouldExtract', () => {
    it('dLsiteドメインで必要な要素がある場合はtrueを返す', () => {
      const hostname = 'play.dlsite.com'
      const rootElement = document.createElement('div')
      rootElement.id = 'root'

      // モックの要素を追加
      const mockElement = document.createElement('div')
      mockElement.setAttribute('data-index', '0')
      document.body.appendChild(mockElement)

      const result = shouldExtract(hostname, rootElement)
      expect(result).toBe(true)

      // クリーンアップ
      document.body.removeChild(mockElement)
    })

    it('dLsiteドメインでない場合はfalseを返す', () => {
      const hostname = 'example.com'
      const rootElement = document.createElement('div')
      rootElement.id = 'root'

      const result = shouldExtract(hostname, rootElement)
      expect(result).toBe(false)
    })

    it('rootElementがnullの場合はfalseを返す', () => {
      const hostname = 'dlsite.com'
      const result = shouldExtract(hostname, null)
      expect(result).toBe(false)
    })

    it('必要な要素がない場合はfalseを返す', () => {
      const hostname = 'dlsite.com'
      const rootElement = document.createElement('div')
      rootElement.id = 'root'

      // data-indexやthumbnail要素を追加しない
      const result = shouldExtract(hostname, rootElement)
      expect(result).toBe(false)
    })
  })

  describe('extractGameDataFromContainer', () => {
    let container: HTMLElement

    beforeEach(() => {
      container = document.createElement('div')
      container.setAttribute('data-index', '0')
    })

    it('正しいゲームデータを抽出する', () => {
      // サムネイル要素を作成
      const thumbnailWrapper = document.createElement('div')
      thumbnailWrapper.className = '_thumbnail_1kd4u_117'
      const thumbnailSpan = document.createElement('span')
      thumbnailSpan.style.backgroundImage = 'url("https://example.com/RJ123456_thumb.jpg")'
      thumbnailWrapper.appendChild(thumbnailSpan)
      container.appendChild(thumbnailWrapper)

      // タイトル要素を作成
      const titleWrapper = document.createElement('div')
      titleWrapper.className = '_workName_1kd4u_192'
      const titleSpan = document.createElement('span')
      titleSpan.textContent = 'Test Game Title'
      titleWrapper.appendChild(titleSpan)
      container.appendChild(titleWrapper)

      // メーカー名要素を作成
      const makerWrapper = document.createElement('div')
      makerWrapper.className = '_makerName_1kd4u_196'
      const makerSpan = document.createElement('span')
      makerSpan.textContent = 'Test Maker'
      makerWrapper.appendChild(makerSpan)
      container.appendChild(makerWrapper)

      const result = extractGameDataFromContainer(container, 0)

      expect(result).toEqual({
        store_id: 'RJ123456',
        title: 'Test Game Title',
        purchase_url: 'https://play.dlsite.com/maniax/work/=/product_id/RJ123456.html',
        purchase_date: '',
        thumbnail_url: 'https://example.com/RJ123456_thumb.jpg',
        additional_data: {
          maker_name: 'Test Maker',
        },
      })
    })

    it('サムネイル要素がない場合はnullを返す', () => {
      const result = extractGameDataFromContainer(container, 0)
      expect(result).toBeNull()
    })

    it('サムネイルURLが不正な場合はnullを返す', () => {
      const thumbnailWrapper = document.createElement('div')
      thumbnailWrapper.className = '_thumbnail_1kd4u_117'
      const thumbnailSpan = document.createElement('span')
      thumbnailSpan.style.backgroundImage = 'none'
      thumbnailWrapper.appendChild(thumbnailSpan)
      container.appendChild(thumbnailWrapper)

      const result = extractGameDataFromContainer(container, 0)
      expect(result).toBeNull()
    })

    it('store_idを抽出できない場合はnullを返す', () => {
      const thumbnailWrapper = document.createElement('div')
      thumbnailWrapper.className = '_thumbnail_1kd4u_117'
      const thumbnailSpan = document.createElement('span')
      thumbnailSpan.style.backgroundImage = 'url("https://example.com/no_code.jpg")'
      thumbnailWrapper.appendChild(thumbnailSpan)
      container.appendChild(thumbnailWrapper)

      const result = extractGameDataFromContainer(container, 0)
      expect(result).toBeNull()
    })

    it('購入日を正しく抽出する', () => {
      // サムネイル要素を作成
      const thumbnailWrapper = document.createElement('div')
      thumbnailWrapper.className = '_thumbnail_1kd4u_117'
      const thumbnailSpan = document.createElement('span')
      thumbnailSpan.style.backgroundImage = 'url("https://example.com/RJ123456_thumb.jpg")'
      thumbnailWrapper.appendChild(thumbnailSpan)
      container.appendChild(thumbnailWrapper)

      // ヘッダー要素を作成（購入日付き）
      const headerWrapper = document.createElement('div')
      headerWrapper.className = '_header_1kd4u_27'
      const headerSpan = document.createElement('span')
      headerSpan.textContent = '購入2024年1月1日'
      headerWrapper.appendChild(headerSpan)
      container.appendChild(headerWrapper)

      const result = extractGameDataFromContainer(container, 0)

      expect(result?.purchase_date).toBe('2024年1月1日')
    })

    it('エラーが発生した場合はnullを返す', () => {
      // 不正な要素構造でエラーを引き起こす
      const thumbnailWrapper = document.createElement('div')
      thumbnailWrapper.className = '_thumbnail_1kd4u_117'
      // spanを追加しないことでquerySelector時にエラーを起こす可能性
      container.appendChild(thumbnailWrapper)

      const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
      const result = extractGameDataFromContainer(container, 0, true)

      expect(result).toBeNull()
      consoleSpy.mockRestore()
    })
  })
})
