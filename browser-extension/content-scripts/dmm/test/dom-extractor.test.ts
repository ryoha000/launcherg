import { beforeEach, describe, expect, it } from 'vitest'
import { extractAllGames, extractGameDataFromContainer, shouldExtract } from '../src/dom-extractor'

describe('dmm dom-extractor', () => {
  describe('shouldExtract', () => {
    it('ターゲットページ条件を満たす場合に true', () => {
      const root = document.createElement('div')
      root.id = 'mylibrary'
      document.body.appendChild(root)
      const ul = document.createElement('ul')
      ul.id = 'js-list'
      ul.innerHTML = '<li></li>'
      document.body.appendChild(ul)
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

  describe('extractGameDataFromContainer', () => {
    let container: HTMLElement
    beforeEach(() => {
      container = document.createElement('li')
      container.innerHTML = `
        <p class="tmb">
          <span class="img"><img src="//img.example.com/a.jpg" alt="ゲームタイトル"></span>
          <span class="txt"><span class="red">【割引】</span> ゲームタイトル</span>
        </p>
        <div class="mylibraryReviewButton"><a href="https://review.dmm.co.jp/create?cid=abc_0001&floor=digital_pcgame"></a></div>
      `
    })

    it('カードから必要項目を抽出できる', () => {
      const g = extractGameDataFromContainer(container, 0)
      expect(g).toEqual({
        store_id: 'abc_0001',
        title: 'ゲームタイトル',
        purchase_url: 'https://dlsoft.dmm.co.jp/mylibrary/?cid=abc_0001',
        purchase_date: '',
        thumbnail_url: '//img.example.com/a.jpg',
        additional_data: {},
      })
    })

    it('cid が取れない場合は null', () => {
      container.querySelector('.mylibraryReviewButton a')!.setAttribute('href', 'https://review.dmm.co.jp/create')
      expect(extractGameDataFromContainer(container, 0)).toBeNull()
    })
  })

  it('extractAllGames で重複除外して配列を返す', () => {
    const ul = document.createElement('ul')
    ul.id = 'js-list'
    ul.innerHTML = `
      <li>
        <p class="tmb"><span class="img"><img src="/a1.jpg" alt="A"></span></p>
        <div class="mylibraryReviewButton"><a href="?cid=x_1"></a></div>
      </li>
      <li>
        <p class="tmb"><span class="img"><img src="/a2.jpg" alt="B"></span></p>
        <div class="mylibraryReviewButton"><a href="?cid=x_1"></a></div>
      </li>
    `
    document.body.appendChild(ul)
    const games = extractAllGames()
    expect(games.length).toBe(1)
    expect(games[0].store_id).toBe('x_1')
    document.body.innerHTML = ''
  })
})
