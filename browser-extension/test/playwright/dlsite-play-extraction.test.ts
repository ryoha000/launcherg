import { test, expect } from '@playwright/test'
import { readFileSync } from 'fs'
import { resolve } from 'path'

// DLsite Play.htmlファイルのパス
const dlsitePlayHtmlPath = resolve(__dirname, '../content-scripts/data/DLsite Play.html')

test.describe('DLsite Play.html パース検証', () => {
  test('実際のDLsite Play.htmlファイルから正しくゲーム情報を抽出できること', async ({ page }) => {
    // HTMLファイルを読み込み
    const htmlContent = readFileSync(dlsitePlayHtmlPath, 'utf-8')
    
    // file:// プロトコルでHTMLを読み込み（データ構造の正確性を保持）
    await page.setContent(htmlContent, { 
      waitUntil: 'load'
    })

    // ページがDLsiteライブラリページとして認識されるように設定
    await page.addInitScript(() => {
      Object.defineProperty(window, 'location', {
        value: {
          hostname: 'play.dlsite.com',
          pathname: '/library',
          href: 'https://play.dlsite.com/library',
          search: ''
        },
        writable: true
      })
    })

    // DLsiteExtractorクラスの核となるパース処理を実行
    const extractedGames = await page.evaluate(() => {
      // ゲームコンテナを検索
      const gameContainers = document.querySelectorAll('[data-index]')
      console.log('Found game containers:', gameContainers.length)
      
      const games: any[] = []
      const seenStoreIds = new Set<string>()

      // extractStoreIdFromUrl関数（DLsiteExtractorと同じロジック）
      function extractStoreIdFromUrl(thumbnailUrl: string): string | null {
        const fileName = thumbnailUrl.split('/').pop() || ''
        const rjMatch = fileName.match(/(RJ|VJ|BJ)([0-9]+)/)
        if (!rjMatch) {
          return null
        }
        return rjMatch[1] + rjMatch[2]
      }

      gameContainers.forEach((container, index) => {
        try {
          // サムネイル要素の検索
          const thumbnailElement = container.querySelector('._thumbnail_1kd4u_117 span') as HTMLElement
          if (!thumbnailElement) {
            return
          }

          // サムネイルURLから情報を抽出
          const bgImage = thumbnailElement.style.backgroundImage
          const thumbnailMatch = bgImage.match(/url\("?(.+?)"?\)/)
          if (!thumbnailMatch) {
            return
          }

          const thumbnailUrl = thumbnailMatch[1]

          // URLからstore_idを抽出
          const storeId = extractStoreIdFromUrl(thumbnailUrl)
          if (!storeId) {
            return
          }

          // 重複チェック
          if (seenStoreIds.has(storeId)) {
            return
          }
          seenStoreIds.add(storeId)

          // タイトルを抽出
          const titleElement = container.querySelector('._workName_1kd4u_192 span')
          const title = titleElement?.textContent?.trim() || ''

          // メーカー名を抽出
          const makerElement = container.querySelector('._makerName_1kd4u_196 span')
          const makerName = makerElement?.textContent?.trim() || ''

          // 購入日を抽出（親要素から探す）
          let purchaseDate = ''
          const headerElement = container.closest('[data-index]')?.querySelector('._header_1kd4u_27 span')
          if (headerElement?.textContent?.includes('購入')) {
            purchaseDate = headerElement.textContent.replace('購入', '').trim()
          }

          const gameData = {
            store_id: storeId,
            title,
            purchase_url: `https://play.dlsite.com/maniax/work/=/product_id/${storeId}.html`,
            purchase_date: purchaseDate,
            thumbnail_url: thumbnailUrl,
            additional_data: {
              maker_name: makerName,
            },
          }

          games.push(gameData)
          console.log(`Extracted game ${index + 1}:`, gameData)
        } catch (error) {
          console.log(`Error extracting game from container ${index}:`, error)
        }
      })

      return {
        games,
        containerCount: gameContainers.length,
        uniqueStoreIds: Array.from(seenStoreIds)
      }
    })

    console.log('Playwright extraction results:', extractedGames)

    // 基本的な検証
    expect(extractedGames.containerCount).toBeGreaterThan(0)
    expect(extractedGames.games.length).toBeGreaterThan(0)
    expect(extractedGames.uniqueStoreIds.length).toBeGreaterThan(0)

    // 各ゲームデータの構造検証
    for (const game of extractedGames.games) {
      expect(game.store_id).toBeTruthy()
      expect(game.store_id).toMatch(/^(RJ|VJ|BJ)\d+$/)
      expect(game.title).toBeTruthy()
      expect(game.purchase_url).toContain('dlsite.com')
      expect(game.thumbnail_url).toContain('img.dlsite.jp')
      
      // メーカー名が存在することを確認
      expect(game.additional_data.maker_name).toBeTruthy()
    }

    // 期待される最小数のゲームが抽出されることを確認
    expect(extractedGames.games.length).toBeGreaterThanOrEqual(3)

    // ログ出力（デバッグ用）
    console.log(`抽出されたゲーム数: ${extractedGames.games.length}`)
    console.log('抽出されたstore_ids:', extractedGames.uniqueStoreIds)
    console.log('最初のゲーム:', extractedGames.games[0])
  })

  test('HTMLにDLsiteの必要な要素が含まれていることを確認する', async ({ page }) => {
    const htmlContent = readFileSync(dlsitePlayHtmlPath, 'utf-8')
    await page.setContent(htmlContent, { waitUntil: 'load' })

    // 必要な要素の存在確認
    const elementCheck = await page.evaluate(() => {
      return {
        hasRoot: !!document.getElementById('root'),
        hasThumbnails: !!document.querySelector('._thumbnail_1kd4u_117'),
        hasDataIndex: !!document.querySelector('[data-index]'),
        dataIndexCount: document.querySelectorAll('[data-index]').length,
        thumbnailCount: document.querySelectorAll('._thumbnail_1kd4u_117 span').length
      }
    })

    expect(elementCheck.hasRoot).toBe(true)
    expect(elementCheck.hasDataIndex).toBe(true)
    expect(elementCheck.hasThumbnails).toBe(true)
    expect(elementCheck.dataIndexCount).toBeGreaterThan(0)
    expect(elementCheck.thumbnailCount).toBeGreaterThan(0)
    
    console.log('Element check results:', elementCheck)
  })

  test('store_id抽出の正確性を個別にテストする', async ({ page }) => {
    const htmlContent = readFileSync(dlsitePlayHtmlPath, 'utf-8')
    await page.setContent(htmlContent, { waitUntil: 'load' })

    // 実際のHTMLから抽出されるサムネイルURLでstore_id抽出をテスト
    const extractionTest = await page.evaluate(() => {
      function extractStoreIdFromUrl(thumbnailUrl: string): string | null {
        const fileName = thumbnailUrl.split('/').pop() || ''
        const rjMatch = fileName.match(/(RJ|VJ|BJ)([0-9]+)/)
        if (!rjMatch) {
          return null
        }
        return rjMatch[1] + rjMatch[2]
      }

      const results: any[] = []
      const thumbnailElements = document.querySelectorAll('._thumbnail_1kd4u_117 span')
      
      thumbnailElements.forEach((element, index) => {
        const htmlElement = element as HTMLElement
        const bgImage = htmlElement.style.backgroundImage
        const match = bgImage.match(/url\("?(.+?)"?\)/)
        
        if (match) {
          const url = match[1]
          const storeId = extractStoreIdFromUrl(url)
          results.push({
            index,
            url,
            storeId,
            valid: storeId ? /^(RJ|VJ|BJ)\d+$/.test(storeId) : false
          })
        }
      })

      return results
    })

    console.log('Store ID extraction test results:', extractionTest)

    // 各抽出結果を検証
    expect(extractionTest.length).toBeGreaterThan(0)
    
    const validExtractions = extractionTest.filter(result => result.valid)
    expect(validExtractions.length).toBeGreaterThan(0)

    // すべての有効なstore_idが正しい形式であることを確認
    for (const result of validExtractions) {
      expect(result.storeId).toMatch(/^(RJ|VJ|BJ)\d+$/)
      expect(result.url).toContain('img.dlsite.jp')
    }
  })
})