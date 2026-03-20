import { describe, expect, it } from 'vitest'
import { extractAllGames, shouldExtract } from '../src/dom-extractor'

describe('dmm dom-extractor', () => {
  describe('shouldExtract', () => {
    it('ターゲットページ条件を満たす場合に true', () => {
      const root = document.createElement('div')
      root.id = 'mylibrary'
      document.body.appendChild(root)
      const img = document.createElement('img')
      img.src = 'https://pics.dmm.co.jp/digital/pcgame/sample/sample.jpg'
      root.appendChild(img)
      expect(shouldExtract('dlsoft.dmm.co.jp', root)).toBe(true)
      document.body.innerHTML = ''
    })

    it('ホスト不一致や要素不足で false', () => {
      const root = document.createElement('div')
      root.id = 'mylibrary'
      expect(shouldExtract('example.com', root)).toBe(false)
      expect(shouldExtract('dlsoft.dmm.co.jp', null)).toBe(false)
      document.body.innerHTML = ''
    })
  })

  // 重複除外は要件外のためテストしない
  it('extractAllGames で画像要素からゲーム情報を抽出できる', () => {
    const root = document.createElement('div')
    root.id = 'mylibrary'
    root.innerHTML = `
      <ul>
        <li>
          <img src="https://pics.dmm.co.jp/digital/pcgame/x_1/x_1ps.jpg" alt="タイトルA" />
        </li>
        <li>
          <img src="https://pics.dmm.co.jp/digital/pcgame/y_2/y_2ps.jpg" alt="タイトルB" />
        </li>
        <li>
          <img src="https://example.com/not-target.jpg" alt="無視される" />
        </li>
        <li>
          <img src="https://pics.dmm.co.jp/digital/pcgame/z_3/z_3ps.jpg" />
        </li>
      </ul>
    `
    document.body.appendChild(root)

    const games = extractAllGames()
    expect(games.length).toBe(2)
    expect(games[0]).toEqual({
      storeId: 'x_1',
      category: 'digital',
      subcategory: 'pcgame',
      title: 'タイトルA',
      imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/x_1/x_1ps.jpg',
    })
    expect(games[1]).toEqual({
      storeId: 'y_2',
      category: 'digital',
      subcategory: 'pcgame',
      title: 'タイトルB',
      imageUrl: 'https://pics.dmm.co.jp/digital/pcgame/y_2/y_2ps.jpg',
    })

    document.body.innerHTML = ''
  })

  it('旧構造では section 配下の画像をレコメンドとして除外する', () => {
    const root = document.createElement('div')
    root.id = 'mylibrary'
    root.innerHTML = `
      <ul>
        <li>
          <img src="https://pics.dmm.co.jp/digital/pcgame/a_1/a_1ps.jpg" alt="A" />
        </li>
      </ul>
      <section>
        <img src="https://pics.dmm.co.jp/digital/pcgame/b_2/b_2ps.jpg" alt="B" />
      </section>
    `
    document.body.appendChild(root)

    const games = extractAllGames()
    expect(games.length).toBe(1)
    expect(games[0].storeId).toBe('a_1')

    document.body.innerHTML = ''
  })

  it('現行構造では productList 配下の画像を section 配下でも抽出する', () => {
    const root = document.createElement('div')
    root.id = 'mylibrary'
    root.innerHTML = `
      <section class="libraryListPage__searchSection">
        <div class="productList">
          <div class="productCard">
            <img src="https://pics.dmm.co.jp/digital/pcgame/current_1/current_1ps.jpg" alt="現在のタイトル" />
          </div>
        </div>
      </section>
    `
    document.body.appendChild(root)

    const games = extractAllGames()
    expect(games.length).toBe(1)
    expect(games[0].storeId).toBe('current_1')
    expect(games[0].title).toBe('現在のタイトル')

    document.body.innerHTML = ''
  })
})
