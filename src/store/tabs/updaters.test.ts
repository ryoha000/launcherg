import type { Tab } from '@/store/tabs'
import { describe, expect, it } from 'vitest'
import { upsertKeyedTab } from '@/store/tabs/updaters'

describe('tabs/updaters: keyed タブの更新', () => {
  it('既存の keyed タブに href と title を反映する', () => {
    const tabs: Tab[] = [
      {
        id: 1,
        workId: '123',
        type: 'works',
        scrollTo: 0,
        title: '旧タイトル',
      },
    ]

    const { nextTabs, selectedIndex } = upsertKeyedTab(tabs, {
      mode: 'keyed',
      type: 'works',
      key: '123',
      title: '新タイトル',
      href: '/works/123?gamename=%E6%96%B0%E3%82%BF%E3%82%A4%E3%83%88%E3%83%AB',
    })

    expect(selectedIndex).toBe(0)
    expect(nextTabs).toHaveLength(1)
    expect(nextTabs[0]).toMatchObject({
      id: 1,
      workId: '123',
      type: 'works',
      title: '新タイトル',
      href: '/works/123?gamename=%E6%96%B0%E3%82%BF%E3%82%A4%E3%83%88%E3%83%AB',
    })
  })
})
