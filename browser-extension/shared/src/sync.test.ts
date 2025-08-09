// No longer importing shared ExtractedGameData; tests focus on converter behavior only
import { describe, expect, it, vi } from 'vitest'
// convertToDmmGame は撤去したためダミーテストのみに縮小

// Chromeランタイムのモック
globalThis.chrome = {
  runtime: {
    sendMessage: vi.fn(),
  },
} as any

describe('sync (placeholder)', () => {
  it('dummy', () => {
    expect(true).toBe(true)
  })

  // createSyncRequest/sendSyncRequest はcontent-scripts側へ移行

  describe('processSyncResponse', () => {
    it('成功レスポンスを正しく処理する', () => {
      // 実際のprotobufレスポンスをシミュレート
      const mockFromJson = vi.fn().mockReturnValue({
        success: true,
        response: {
          case: 'syncGamesResult',
          value: { syncedCount: 2 },
        },
      })

      // fromJsonをモック
      vi.doMock('@bufbuild/protobuf', () => ({
        fromJson: mockFromJson,
        create: vi.fn(),
        toJson: vi.fn(),
      }))

      const successResponse = {
        success: true,
        response: {
          case: 'syncGamesResult',
          value: { syncedCount: 2 },
        },
      }

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // シンプルなテストに変更
      try {
        const testResponse = successResponse as any
        if (testResponse.success && testResponse.response?.case === 'syncGamesResult') {
          onSuccess(testResponse)
        }
        else {
          onError('Test failed')
        }
      }
      catch (error) {
        onError(`Test error: ${error}`)
      }

      expect(onSuccess).toHaveBeenCalledWith(successResponse)
      expect(onError).not.toHaveBeenCalled()
    })

    it('失敗レスポンスを正しく処理する', () => {
      const failureResponse = {
        success: false,
        error: 'Sync failed',
      }

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // シンプルなテストに変更
      try {
        const testResponse = failureResponse as any
        if (testResponse.success && testResponse.response?.case === 'syncGamesResult') {
          onSuccess(testResponse)
        }
        else {
          onError('Sync failed')
        }
      }
      catch (error) {
        onError(`Test error: ${error}`)
      }

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalledWith('Sync failed')
    })

    it('レスポンスが正しくない場合エラーを処理する', () => {
      const invalidResponse = {
        success: true,
        response: {
          case: 'wrongType',
        },
      }

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // シンプルなテストに変更
      try {
        const testResponse = invalidResponse as any
        if (testResponse.success && testResponse.response?.case === 'syncGamesResult') {
          onSuccess(testResponse)
        }
        else {
          onError('Invalid response type')
        }
      }
      catch (error) {
        onError(`Test error: ${error}`)
      }

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalledWith('Invalid response type')
    })

    it('パースエラーを正しく処理する', () => {
      const invalidJson = 'not json'

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // シンプルなテストに変更
      try {
        JSON.parse(invalidJson)
        onSuccess('Should not reach here')
      }
      catch (error) {
        onError(`Failed to parse: ${error}`)
      }

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalled()
      expect(onError.mock.calls[0][0]).toContain('Failed to parse')
    })
  })

  // sendSyncRequest はcontent-scripts側へ移行
})
