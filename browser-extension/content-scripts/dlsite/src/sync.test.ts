import type { ExtractedGameData } from './types'
import { describe, expect, it, vi } from 'vitest'
import {
  convertToGameDataSchema,
  createSyncRequest,
  processSyncResponse,
  sendSyncRequest,
} from './sync'

// Chromeランタイムのモック
globalThis.chrome = {
  runtime: {
    sendMessage: vi.fn(),
  },
} as any

describe('sync', () => {
  describe('convertToGameDataSchema', () => {
    it('ゲームデータを正しくProtobufメッセージに変換する', () => {
      const gameData: ExtractedGameData = {
        store_id: 'RJ123456',
        title: 'Test Game',
        purchase_url: 'https://example.com/game',
        purchase_date: '2024-01-15',
        thumbnail_url: 'https://example.com/thumb.jpg',
        additional_data: {
          maker_name: 'Test Maker',
          work_type: 'doujin',
        },
      }

      const result = convertToGameDataSchema(gameData)

      expect(result.storeId).toBe('RJ123456')
      expect(result.title).toBe('Test Game')
      expect(result.purchaseUrl).toBe('https://example.com/game')
      expect(result.purchaseDate).toBe('2024-01-15')
      expect(result.thumbnailUrl).toBe('https://example.com/thumb.jpg')
      expect(result.additionalData).toEqual({
        maker_name: 'Test Maker',
        work_type: 'doujin',
      })
    })

    it('オプショナルフィールドが未定義の場合も正しく処理する', () => {
      const gameData: ExtractedGameData = {
        store_id: 'RJ123456',
        title: 'Test Game',
        purchase_url: 'https://example.com/game',
        additional_data: {},
      }

      const result = convertToGameDataSchema(gameData)

      expect(result.storeId).toBe('RJ123456')
      expect(result.title).toBe('Test Game')
      expect(result.purchaseUrl).toBe('https://example.com/game')
      expect(result.purchaseDate).toBe('')
      expect(result.thumbnailUrl).toBe('')
      expect(result.additionalData).toEqual({})
    })
  })

  describe('createSyncRequest', () => {
    it('同期リクエストを正しく作成する', () => {
      const games: ExtractedGameData[] = [
        {
          store_id: 'RJ123456',
          title: 'Game 1',
          purchase_url: 'https://example.com/game1',
          additional_data: {},
        },
        {
          store_id: 'VJ987654',
          title: 'Game 2',
          purchase_url: 'https://example.com/game2',
          additional_data: {},
        },
      ]

      const request = createSyncRequest(games)

      expect(request.requestId).toBeTruthy()
      expect(request.request.case).toBe('syncGames')
      expect(request.request.value.store).toBe('DLSite')
      expect(request.request.value.source).toBe('dlsite-extractor')
      expect(request.request.value.games).toHaveLength(2)
      expect(request.request.value.games[0].storeId).toBe('RJ123456')
      expect(request.request.value.games[1].storeId).toBe('VJ987654')
    })

    it('空の配列でも正しく処理する', () => {
      const request = createSyncRequest([])

      expect(request.requestId).toBeTruthy()
      expect(request.request.case).toBe('syncGames')
      expect(request.request.value.games).toHaveLength(0)
    })
  })

  describe('processSyncResponse', () => {
    it('成功レスポンスを正しく処理する', () => {
      const successResponse = {
        success: true,
        response: {
          case: 'syncGamesResult',
          value: { syncedCount: 2 },
        },
      }

      const onSuccess = vi.fn()
      const onError = vi.fn()

      processSyncResponse(successResponse, onSuccess, onError)

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

      processSyncResponse(failureResponse, onSuccess, onError)

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalledWith(`Sync failed: ${JSON.stringify(failureResponse)}`)
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

      processSyncResponse(invalidResponse, onSuccess, onError)

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalledWith(`Sync failed: ${JSON.stringify(invalidResponse)}`)
    })

    it('パースエラーを正しく処理する', () => {
      const invalidJson = 'not json'

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // fromJsonのモックが必要な場合の処理
      // この例では、processSyncResponseの実装を前提としている
      processSyncResponse(invalidJson, onSuccess, onError)

      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalled()
      expect(onError.mock.calls[0][0]).toContain('Failed to parse sync response')
    })
  })

  describe('sendSyncRequest', () => {
    it('同期リクエストを送信し、成功レスポンスを処理する', () => {
      const games: ExtractedGameData[] = [
        {
          store_id: 'RJ123456',
          title: 'Test Game',
          purchase_url: 'https://example.com/game',
          additional_data: {},
        },
      ]

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // sendMessageのモックをセットアップ
      const mockSendMessage = vi.fn((message, callback) => {
        // 成功レスポンスをシミュレート
        callback({
          success: true,
          response: {
            case: 'syncGamesResult',
            value: { syncedCount: 1 },
          },
        })
      })
      globalThis.chrome.runtime.sendMessage = mockSendMessage

      sendSyncRequest(games, onSuccess, onError)

      expect(mockSendMessage).toHaveBeenCalled()
      expect(onSuccess).toHaveBeenCalled()
      expect(onError).not.toHaveBeenCalled()
    })

    it('同期リクエストを送信し、エラーレスポンスを処理する', () => {
      const games: ExtractedGameData[] = [
        {
          store_id: 'RJ123456',
          title: 'Test Game',
          purchase_url: 'https://example.com/game',
          additional_data: {},
        },
      ]

      const onSuccess = vi.fn()
      const onError = vi.fn()

      // sendMessageのモックをセットアップ
      const mockSendMessage = vi.fn((message, callback) => {
        // エラーレスポンスをシミュレート
        callback({
          success: false,
          error: 'Network error',
        })
      })
      globalThis.chrome.runtime.sendMessage = mockSendMessage

      sendSyncRequest(games, onSuccess, onError)

      expect(mockSendMessage).toHaveBeenCalled()
      expect(onSuccess).not.toHaveBeenCalled()
      expect(onError).toHaveBeenCalled()
    })
  })
})
