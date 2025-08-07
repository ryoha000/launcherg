import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { addNotificationStyles, createNotificationRequest, showInPageNotification } from './notification'

// Chromeランタイムのモック
globalThis.chrome = {
  runtime: {
    sendMessage: vi.fn(),
  },
} as any

describe('notification', () => {
  describe('showInPageNotification', () => {
    beforeEach(() => {
      // タイマーのモック
      vi.useFakeTimers()
    })

    afterEach(() => {
      // DOMのクリーンアップ
      document.body.innerHTML = ''
      vi.useRealTimers()
    })

    it('成功通知を正しく表示する', () => {
      showInPageNotification('テストメッセージ', 'success')

      const notification = document.querySelector('div') as HTMLDivElement
      expect(notification).toBeTruthy()
      expect(notification.textContent).toBe('テストメッセージ')
      expect(notification.style.background).toContain('#4CAF50')
    })

    it('エラー通知を正しく表示する', () => {
      showInPageNotification('エラーメッセージ', 'error')

      const notification = document.querySelector('div') as HTMLDivElement
      expect(notification).toBeTruthy()
      expect(notification.textContent).toBe('エラーメッセージ')
      expect(notification.style.background).toContain('#f44336')
    })

    it('4秒後に通知を自動削除する', () => {
      showInPageNotification('一時的なメッセージ', 'success')

      let notification = document.querySelector('div')
      expect(notification).toBeTruthy()

      // 3秒進める（まだ表示されているはず）
      vi.advanceTimersByTime(3000)
      notification = document.querySelector('div')
      expect(notification).toBeTruthy()

      // さらに1秒進める（削除されるはず）
      vi.advanceTimersByTime(1000)
      notification = document.querySelector('div')
      expect(notification).toBeFalsy()
    })

    it('正しいスタイルが適用される', () => {
      showInPageNotification('スタイルテスト', 'success')

      const notification = document.querySelector('div') as HTMLDivElement
      expect(notification.style.position).toBe('fixed')
      expect(notification.style.top).toBe('20px')
      expect(notification.style.right).toBe('20px')
      expect(notification.style.zIndex).toBe('10000')
      expect(notification.style.padding).toBe('12px 20px')
      expect(notification.style.borderRadius).toBe('4px')
    })
  })

  describe('addNotificationStyles', () => {
    afterEach(() => {
      // スタイルタグのクリーンアップ
      document.head.innerHTML = ''
    })

    it('アニメーションスタイルを追加する', () => {
      addNotificationStyles()

      const style = document.querySelector('style')
      expect(style).toBeTruthy()
      expect(style?.textContent).toContain('@keyframes slideIn')
      expect(style?.textContent).toContain('transform: translateX(100%)')
      expect(style?.textContent).toContain('transform: translateX(0)')
    })
  })

  describe('createNotificationRequest', () => {
    it('成功通知のリクエストを正しく作成する', () => {
      const request = createNotificationRequest('成功メッセージ', 'success')

      expect(request.requestId).toBeTruthy()
      expect(request.request.case).toBe('showNotification')
      expect(request.request.value.title).toBe('Launcherg DL Store Sync')
      expect(request.request.value.message).toBe('成功メッセージ')
      expect(request.request.value.iconType).toBe('success')
    })

    it('エラー通知のリクエストを正しく作成する', () => {
      const request = createNotificationRequest('エラーメッセージ', 'error')

      expect(request.requestId).toBeTruthy()
      expect(request.request.case).toBe('showNotification')
      expect(request.request.value.title).toBe('Launcherg DL Store Sync')
      expect(request.request.value.message).toBe('エラーメッセージ')
      expect(request.request.value.iconType).toBe('error')
    })

    it('毎回異なるrequestIdを生成する', () => {
      const request1 = createNotificationRequest('メッセージ1', 'success')
      const request2 = createNotificationRequest('メッセージ2', 'success')

      expect(request1.requestId).toBeTruthy()
      expect(request2.requestId).toBeTruthy()
      expect(request1.requestId).not.toBe(request2.requestId)
    })
  })
})
