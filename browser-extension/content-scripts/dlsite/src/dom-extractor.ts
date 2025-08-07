// DOM操作関連の純粋関数

import type { ExtractedGameData } from '@launcherg/shared'
import { extractStoreIdFromUrl } from './utils'

// ページが抽出対象かどうかを判定する純粋関数
export function shouldExtract(hostname: string, rootElement: HTMLElement | null): boolean {
  // ページURL確認
  if (!hostname.includes('dlsite.com')) {
    return false
  }

  // 新しいDLsiteのReactベースのページを検出
  const hasLibraryContent
    = rootElement
      && (document.querySelector('._thumbnail_1kd4u_117') !== null
        || document.querySelector('[data-index]') !== null)

  return !!hasLibraryContent
}

// ゲームコンテナー要素を取得する純粋関数
export function extractGameContainers(): NodeListOf<Element> {
  return document.querySelectorAll('[data-index]')
}

// コンテナー要素からゲームデータを抽出する純粋関数
export function extractGameDataFromContainer(
  container: Element,
  index: number,
  debugMode: boolean = false,
): ExtractedGameData | null {
  try {
    // 実際のゲームアイテムかどうかを確認（サムネイルがあるか）
    const thumbnailElement = container.querySelector(
      '._thumbnail_1kd4u_117 span',
    ) as HTMLElement
    if (!thumbnailElement) {
      return null
    }

    // サムネイルURLから情報を抽出
    const bgImage = thumbnailElement.style.backgroundImage
    const thumbnailMatch = bgImage.match(/url\("?(.+?)"\)/)
    if (!thumbnailMatch) {
      return null
    }

    const thumbnailUrl = thumbnailMatch[1]

    // URLからstore_idを抽出
    const storeId = extractStoreIdFromUrl(thumbnailUrl)
    if (debugMode) {
      console.log(`[DLsite Extractor] Extracted store_id "${storeId}" from URL: ${thumbnailUrl}`)
    }
    if (!storeId) {
      return null
    }

    // タイトルを抽出
    const titleElement = container.querySelector(
      '._workName_1kd4u_192 span',
    )
    const title = titleElement?.textContent?.trim() || ''

    // メーカー名を抽出
    const makerElement = container.querySelector(
      '._makerName_1kd4u_196 span',
    )
    const makerName = makerElement?.textContent?.trim() || ''

    // 購入日を抽出（親要素から探す）
    let purchaseDate = ''
    const headerElement = container
      .closest('[data-index]')
      ?.querySelector('._header_1kd4u_27 span')
    if (headerElement?.textContent?.includes('購入')) {
      purchaseDate = headerElement.textContent.replace('購入', '').trim()
    }

    // 購入URLを構築
    const purchaseUrl = `https://play.dlsite.com/maniax/work/=/product_id/${storeId}.html`

    const gameData: ExtractedGameData = {
      store_id: storeId,
      title,
      purchase_url: purchaseUrl,
      purchase_date: purchaseDate,
      thumbnail_url: thumbnailUrl,
      additional_data: {
        maker_name: makerName,
      },
    }

    if (debugMode) {
      console.log(`[DLsite Extractor] Extracted game ${index + 1}:`, gameData)
    }

    return gameData
  }
  catch (error) {
    if (debugMode) {
      console.log(`[DLsite Extractor] Error extracting game from container ${index}:`, error)
    }
    return null
  }
}

// すべてのゲームデータを抽出する純粋関数
export function extractAllGames(debugMode: boolean = false): ExtractedGameData[] {
  const gameContainers = extractGameContainers()
  if (debugMode) {
    console.log(`[DLsite Extractor] Found ${gameContainers.length} potential game containers`)
  }

  const games: ExtractedGameData[] = []
  const seenStoreIds = new Set<string>()

  gameContainers.forEach((container, index) => {
    const gameData = extractGameDataFromContainer(container, index, debugMode)

    if (gameData) {
      // 重複チェック
      if (!seenStoreIds.has(gameData.store_id)) {
        seenStoreIds.add(gameData.store_id)
        games.push(gameData)
      }
    }
  })

  return games
}
