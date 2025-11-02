import type { Descriptor } from '@/router/registry'
import { describe, expect, it } from 'vitest'
import {
  buildPath,
  getTabActionFromLocation,
  keyedTab,
  pathParamExtractor,
  queryParamExtractor,
  singletonTab,
} from '@/store/tabs/schema'

function d(partial: Omit<Descriptor, 'component'>): Descriptor<any> {
  return { ...partial, component: {} }
}

describe('tabs/schema: 基本ヘルパ', () => {
  it('pathParamExtractor: パスパラメータから値を取得できる', () => {
    const ex = pathParamExtractor('id')
    expect(ex({ path: '/x', pathParams: { id: '42' } })).toBe('42')
    expect(ex({ path: '/x', pathParams: { id: 7 } })).toBe('7')
    expect(ex({ path: '/x', pathParams: {} })).toBeUndefined()
  })

  it('queryParamExtractor: クエリから値を取得できる', () => {
    const ex = queryParamExtractor('title')
    expect(ex({ path: '/x', queryParams: { title: 'ゲーム' } })).toBe('ゲーム')
    expect(ex({ path: '/x', queryParams: {} })).toBeUndefined()
  })

  it('singletonTab/keyedTab: ポリシーが正しく生成される', () => {
    const s = singletonTab('設定')
    expect(s).toEqual({ mode: 'singleton', title: '設定' })

    const k = keyedTab(pathParamExtractor('id'), queryParamExtractor('title'))
    expect(k.mode).toBe('keyed')
    if (k.mode !== 'keyed')
      throw new Error('expected keyed')
    expect(typeof k.key).toBe('function')
  })
})

describe('tabs/schema: getTabActionFromLocation', () => {
  it('mode:none のルートは none を返す', () => {
    const reg: readonly Descriptor[] = [
      d({ kind: 'home', pathTemplate: '/', tab: { mode: 'none' } }),
    ]
    const act = getTabActionFromLocation(reg, { path: '/' })
    expect(act).toEqual({ mode: 'none' })
  })

  it('singleton はタイトルとともに返る', () => {
    const reg: readonly Descriptor[] = [
      d({ kind: 'settings', pathTemplate: '/settings', tab: singletonTab('設定') }),
    ]
    const act = getTabActionFromLocation(reg, { path: '/settings' })
    expect(act).toEqual({ mode: 'singleton', type: 'settings', title: '設定' })
  })

  it('keyed: パスからキー抽出し、必要ならタイトル抽出も行う（editor あり）', () => {
    const reg: readonly Descriptor[] = [
      d({
        kind: 'works',
        pathTemplate: '/works/:id(\\d+)',
        tab: keyedTab(pathParamExtractor('id'), queryParamExtractor('title')),
      }),
    ]
    const act = getTabActionFromLocation(reg, {
      path: '/works/123',
      queryParams: { title: 'ゲームA' },
    })
    expect(act).toEqual({ mode: 'keyed', type: 'works', key: '123', title: 'ゲームA' })
  })

  it('keyed: extractor 側で decode され、editor は加工のみ行う', () => {
    const reg: readonly Descriptor[] = [
      d({
        kind: 'memos',
        pathTemplate: '/memos/:id(\\d+)',
        tab: keyedTab(pathParamExtractor('id'), queryParamExtractor('gamename', v => `メモ - ${v}`)),
      }),
    ]
    const act = getTabActionFromLocation(reg, {
      path: '/memos/7',
      queryParams: { gamename: encodeURIComponent('タイトル') },
    })
    expect(act).toEqual({ mode: 'keyed', type: 'memos', key: '7', title: 'メモ - タイトル' })
  })

  it('keyed: キーが取得できなければ none', () => {
    const reg: readonly Descriptor[] = [
      d({
        kind: 'q',
        pathTemplate: '/q',
        tab: keyedTab(({ queryParams }) => queryParams?.id as any),
      }),
    ]
    const act = getTabActionFromLocation(reg, { path: '/q' })
    expect(act).toEqual({ mode: 'none' })
  })
})

describe('tabs/schema: buildPath', () => {
  it('singleton/none はテンプレートそのまま（正規化）を返す', () => {
    const s = d({ kind: 'settings', pathTemplate: '/settings', tab: singletonTab('設定') })
    const n = d({ kind: 'home', pathTemplate: '/', tab: { mode: 'none' } })
    expect(buildPath(s)).toBe('/settings')
    expect(buildPath(n)).toBe('/')
  })

  it('keyed は :id(...) を埋めて返す', () => {
    const k = d({
      kind: 'works',
      pathTemplate: '/works/:id(\\d+)',
      tab: keyedTab(pathParamExtractor('id')),
    })
    expect(buildPath(k, '456')).toBe('/works/456')
    expect(buildPath(k)).toBe('/works/:id(\\d+)')
  })
})
