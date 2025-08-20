import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { describe, expect, it } from 'vitest'
import { createEgsResolver } from './resolver'

// 実ネットワークにアクセスする統合テスト。
// CI やローカルの通常実行では外部リクエストを避けるため skip します。
describe.skip('eGS リゾルバ（実ネットワーク）', () => {
  const resolver = createEgsResolver()

  it('dMM: resolveForDmm は HTML を解析して期待どおりの EgsInfo を返す（内容検証）', async () => {
    const storeId = 'feng_0004'
    const category = 'digital'
    const subcategory = 'pcgame'
    // select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where dmm = 'feng_0004' and dmm_genre = 'digital' and dmm_genre_2 = 'pcgame' limit 1;

    const result = await resolver.resolveForDmm(storeId, category, subcategory)

    const expected = create(EgsInfoSchema, {
      erogamescapeId: 21641,
      gamename: '妹のセイイキ',
      gamenameRuby: 'イモウトノセイイキ',
      brandname: 'feng',
      brandnameRuby: 'フォン',
      sellday: '2015-08-28',
      isNukige: true,
    })
    expect(result).toEqual(expected)
  }, 30_000)

  it('dLsite: resolveForDlsite は HTML を解析して期待どおりの EgsInfo を返す（内容検証）', async () => {
    const storeId = 'RJ278019'
    const category = 'maniax'
    // select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where dlsite_id = 'RJ278019' and dlsite_domain = 'maniax' limit 1;

    const result = await resolver.resolveForDlsite(storeId, category)

    const expected = create(EgsInfoSchema, {
      erogamescapeId: 29245,
      gamename: 'MECHANICA -うさぎと水星のバラッド-',
      gamenameRuby: 'メカニカウサギトスイセイノバラッド',
      brandname: 'Loser/s',
      brandnameRuby: 'ルーザーハイフンエス',
      sellday: '2020-04-10',
      isNukige: true,
    })
    expect(result).toEqual(expected)
  }, 30_000)

  it('dMM: resolveForDmmBulk は複数キーの結果を配列順で返す（内容検証・ヒット/ミス混在）', async () => {
    const items = [
      { storeId: 'feng_0004', category: 'digital', subcategory: 'pcgame' },
      { storeId: 'abgktk_0010', category: 'digital', subcategory: 'pcgame' },
      { storeId: 'dummy_id', category: 'digital', subcategory: 'pcgame' },
    ]
    // select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where dmm = 'feng_0004' and dmm_genre = 'digital' and dmm_genre_2 = 'pcgame' limit 1;
    const results = await resolver.resolveForDmmBulk(items)
    const expected0 = create(EgsInfoSchema, {
      erogamescapeId: 21641,
      gamename: '妹のセイイキ',
      gamenameRuby: 'イモウトノセイイキ',
      brandname: 'feng',
      brandnameRuby: 'フォン',
      sellday: '2015-08-28',
      isNukige: true,
    })
    const expected1 = create(EgsInfoSchema, {
      erogamescapeId: 13640,
      gamename: '穢翼のユースティア',
      gamenameRuby: 'アイヨクノユースティア',
      brandname: 'AUGUST',
      brandnameRuby: 'オーガスト',
      sellday: '2011-04-28',
      isNukige: false,
    })

    expect(Array.isArray(results)).toBe(true)
    expect(results).toHaveLength(items.length)
    expect(results[0]).toEqual(expected0)
    expect(results[1]).toEqual(expected1)
    expect(results[2]).toBeNull()
  }, 30_000)

  it('dLsite: resolveForDlsiteBulk は複数キーの結果を配列順で返す（内容検証・ヒット/ミス混在）', async () => {
    const items = [
      { storeId: 'RJ278019', category: 'maniax' },
      { storeId: 'RJ000000', category: 'maniax' },
    ]
    const results = await resolver.resolveForDlsiteBulk(items)
    const expected0 = create(EgsInfoSchema, {
      erogamescapeId: 29245,
      gamename: 'MECHANICA -うさぎと水星のバラッド-',
      gamenameRuby: 'メカニカウサギトスイセイノバラッド',
      brandname: 'Loser/s',
      brandnameRuby: 'ルーザーハイフンエス',
      sellday: '2020-04-10',
      isNukige: true,
    })
    expect(Array.isArray(results)).toBe(true)
    expect(results).toHaveLength(items.length)
    expect(results[0]).toEqual(expected0)
    expect(results[1]).toBeNull()
  }, 30_000)
})
