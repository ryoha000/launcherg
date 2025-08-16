import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createEgsResolver } from './resolver'

function htmlTable(rows: string[][]): string {
  const header = '<tr><th>h1</th></tr>'
  const body = rows.map(cols => `<tr>${cols.map(c => `<td>${c}</td>`).join('')}</tr>`).join('')
  return `<html><body><table>${header}${body}</table></body></html>`
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('eGS リゾルバ（モック）', () => {
  beforeEach(() => {
    const mem: Record<string, any> = {}
    ;(chrome.storage.local.get as any).mockImplementation((keys: any, cb: (items: Record<string, any>) => void) => {
      if (Array.isArray(keys)) {
        const res: Record<string, any> = {}
        for (const k of keys)
          res[k] = mem[k]
        cb(res)
      }
      else if (typeof keys === 'string') {
        cb({ [keys]: mem[keys] })
      }
      else {
        cb(mem)
      }
    })
    ;(chrome.storage.local.set as any).mockImplementation((items: Record<string, any>, cb?: () => void) => {
      Object.assign(mem, items)
      cb && cb()
    })
  })
  it('dMM: resolveForDmm は HTML を解析して EgsInfo を返す', async () => {
    const row = ['21641', '妹のセイイキ', 'イモウトノセイイキ', '2015-08-28', 't', 'feng', 'フォン']
    vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      text: async () => htmlTable([row]),
    } as unknown as Response)

    const resolver = createEgsResolver()
    const result = await resolver.resolveForDmm('feng_0004', 'digital', 'pcgame')

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
    expect(globalThis.fetch).toHaveBeenCalledTimes(1)
  })

  it('dLsite: resolveForDlsite は HTML を解析して EgsInfo を返す', async () => {
    const row = ['29245', 'MECHANICA -うさぎと水星のバラッド-', 'メカニカウサギトスイセイノバラッド', '2020-04-10', 't', 'Loser/s', 'ルーザーハイフンエス']
    vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      text: async () => htmlTable([row]),
    } as unknown as Response)

    const resolver = createEgsResolver()
    const result = await resolver.resolveForDlsite('RJ278019', 'maniax')

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
    expect(globalThis.fetch).toHaveBeenCalledTimes(1)
  })

  it('dMM: resolveForDmmBulk は複数キーの結果を配列順で返す（ヒット/ミス混在）', async () => {
    const rows = [[
      '21641',
      '妹のセイイキ',
      'イモウトノセイイキ',
      '2015-08-28',
      't',
      'feng',
      'フォン',
      'feng_0004',
      'digital',
      'pcgame',
    ]]
    vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      text: async () => htmlTable(rows),
    } as unknown as Response)

    const resolver = createEgsResolver()
    const items = [
      { storeId: 'feng_0004', category: 'digital', subcategory: 'pcgame' },
      { storeId: 'dummy', category: 'digital', subcategory: 'pcgame' },
    ]
    const results = await resolver.resolveForDmmBulk!(items)

    const expected0 = create(EgsInfoSchema, {
      erogamescapeId: 21641,
      gamename: '妹のセイイキ',
      gamenameRuby: 'イモウトノセイイキ',
      brandname: 'feng',
      brandnameRuby: 'フォン',
      sellday: '2015-08-28',
      isNukige: true,
    })

    expect(results).toHaveLength(2)
    expect(results[0]).toEqual(expected0)
    expect(results[1]).toBeNull()
    expect(globalThis.fetch).toHaveBeenCalledTimes(1)
  })

  it('dLsite: resolveForDlsiteBulk は複数キーの結果を配列順で返す（ヒット/ミス混在）', async () => {
    const rows = [[
      '29245',
      'MECHANICA -うさぎと水星のバラッド-',
      'メカニカウサギトスイセイノバラッド',
      '2020-04-10',
      't',
      'Loser/s',
      'ルーザーハイフンエス',
      'RJ278019',
      'maniax',
    ]]
    vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      text: async () => htmlTable(rows),
    } as unknown as Response)

    const resolver = createEgsResolver()
    const items = [
      { storeId: 'RJ278019', category: 'maniax' },
      { storeId: 'RJ000000', category: 'maniax' },
    ]
    const results = await resolver.resolveForDlsiteBulk!(items)

    const expected0 = create(EgsInfoSchema, {
      erogamescapeId: 29245,
      gamename: 'MECHANICA -うさぎと水星のバラッド-',
      gamenameRuby: 'メカニカウサギトスイセイノバラッド',
      brandname: 'Loser/s',
      brandnameRuby: 'ルーザーハイフンエス',
      sellday: '2020-04-10',
      isNukige: true,
    })

    expect(results).toHaveLength(2)
    expect(results[0]).toEqual(expected0)
    expect(results[1]).toBeNull()
    expect(globalThis.fetch).toHaveBeenCalledTimes(1)
  })

  it('dMM: キャッシュヒット時は fetch を呼ばない', async () => {
    const row = ['21641', '妹のセイイキ', 'イモウトノセイイキ', '2015-08-28', 't', 'feng', 'フォン']
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      text: async () => htmlTable([row]),
    } as unknown as Response)

    const resolver = createEgsResolver()
    const key = { storeId: 'feng_0004', category: 'digital', subcategory: 'pcgame' }
    const first = await resolver.resolveForDmm(key.storeId, key.category, key.subcategory)
    expect(first).not.toBeNull()

    // 2 回目はキャッシュから返ってくるので fetch は呼ばれない
    const second = await resolver.resolveForDmm(key.storeId, key.category, key.subcategory)
    expect(second).toEqual(first)
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })
})
