// 同期処理関連の関数

import type { ExtractedGameData } from './types'
import { create, fromJson, toJson } from '@bufbuild/protobuf'
import {
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GameDataSchema,
  SyncGamesRequestSchema,
} from './proto/extension_internal'
import { generateRequestId } from './utils'

// ゲームデータをProtobufメッセージに変換する純粋関数
export function convertToGameDataSchema(game: ExtractedGameData): any {
  return create(GameDataSchema, {
    storeId: game.store_id,
    title: game.title,
    purchaseUrl: game.purchase_url,
    purchaseDate: game.purchase_date || '',
    thumbnailUrl: game.thumbnail_url || '',
    additionalData: game.additional_data,
  })
}

// 同期リクエストを作成する純粋関数
export function createSyncRequest(storeName: string, games: ExtractedGameData[], source: string): any {
  const gameDataList = games.map(game => convertToGameDataSchema(game))

  return create(ExtensionRequestSchema, {
    requestId: generateRequestId(),
    request: {
      case: 'syncGames',
      value: create(SyncGamesRequestSchema, {
        store: storeName,
        games: gameDataList,
        source,
      }),
    },
  })
}

// 同期レスポンスを処理する関数
export function processSyncResponse(
  responseJson: any,
  onSuccess: (response: any) => void,
  onError: (error: string) => void,
): void {
  try {
    const response = fromJson(ExtensionResponseSchema, responseJson)
    if (
      response
      && response.success
      && response.response.case === 'syncGamesResult'
    ) {
      onSuccess(response)
    }
    else {
      onError(`Sync failed: ${JSON.stringify(response)}`)
    }
  }
  catch (error) {
    onError(`Failed to parse sync response: ${error}`)
  }
}

// 同期リクエストを送信する関数
export function sendSyncRequest(
  storeName: string,
  games: ExtractedGameData[],
  source: string,
  onSuccess: (response: any) => void,
  onError: (error: string) => void,
): void {
  const syncRequest = createSyncRequest(storeName, games, source)

  chrome.runtime.sendMessage(
    toJson(ExtensionRequestSchema, syncRequest),
    (responseJson) => {
      processSyncResponse(responseJson, onSuccess, onError)
    },
  )
}
