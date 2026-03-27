import type { SidebarWorkItem } from '@/store/sidebarWorks'
import { describe, expect, it } from 'vitest'
import {
  ATTRIBUTES,
  FILTER_BY_ATTRIBUTE,
  mergeMissingAttributes,
} from '@/components/Sidebar/searchAttributes'

describe('sidebar search attributes', () => {
  it('既存の保存状態に新しい属性を補完する', () => {
    const attrs = mergeMissingAttributes([
      { key: ATTRIBUTES.LIKE, enabled: true },
      { key: ATTRIBUTES.DMM, enabled: false },
    ])

    expect(attrs).toEqual([
      { key: ATTRIBUTES.LIKE, enabled: true },
      { key: ATTRIBUTES.DMM, enabled: false },
      { key: ATTRIBUTES.DLSITE, enabled: false },
      { key: ATTRIBUTES.INSTALL, enabled: false },
      { key: ATTRIBUTES.NUKIGE, enabled: false },
      { key: ATTRIBUTES.NOT_NUKIGE, enabled: false },
      { key: ATTRIBUTES.EXIST_PATH, enabled: false },
    ])
  })

  it('dmm / dlsite / インストール済みで絞り込める', () => {
    const items: SidebarWorkItem[] = [
      { id: '1', title: 'A', dmm: { id: 1, storeId: 'a', category: 'pcgame' } },
      { id: '2', title: 'B', dlsite: { id: 2, storeId: 'b', category: 'doujin' } },
      { id: '3', title: 'C', installAt: '2024-01-01' },
      { id: '4', title: 'D' },
    ]

    expect(FILTER_BY_ATTRIBUTE.dmm(items)).toEqual([items[0]])
    expect(FILTER_BY_ATTRIBUTE.dlsite(items)).toEqual([items[1]])
    expect(FILTER_BY_ATTRIBUTE.install(items)).toEqual([items[2]])
  })
})
