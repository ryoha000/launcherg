import type { Descriptor } from '@/router/registry'
import type { Tab } from '@/store/tabs'
import { describe, expect, it } from 'vitest'
import { keyedTab, pathParamExtractor, singletonTab } from '@/store/tabs/schema'
import { computeTabDeletionPlan } from '@/store/tabs/updaters'

function t(id: number, type: string, workId: string): Tab {
  return { id, type, workId, scrollTo: 0, title: `${type}-${workId}` }
}

describe('tabs/delete: 削除後の遷移・選択の決定', () => {
  const REG: readonly Descriptor[] = [
    { kind: 'home', pathTemplate: '/', component: {}, tab: { mode: 'none' } },
    { kind: 'settings', pathTemplate: '/settings', component: {}, tab: singletonTab('設定') },
    { kind: 'store-mapped', pathTemplate: '/store-mapped', component: {}, tab: singletonTab('管理') },
    { kind: 'works', pathTemplate: '/works/:id(\\d+)', component: {}, tab: keyedTab(pathParamExtractor('id')) },
  ]

  it('1枚のみを削除すると home へ遷移', () => {
    const tabs = [t(1, 'settings', '')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 0, deleteId: 1, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBe('/')
    expect(plan.selectedIndexAfterDelete).toBeNull()
  })

  it('存在しないIDは何もしない', () => {
    const tabs = [t(1, 'settings', ''), t(2, 'store-mapped', '')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 0, deleteId: 999, registry: REG })
    expect(plan.shouldDelete).toBe(false)
    expect(plan.navigateToPath).toBeNull()
  })

  it('非選択を削除しても遷移しない（選択は維持）', () => {
    const tabs = [t(1, 'settings', ''), t(2, 'store-mapped', ''), t(3, 'settings', '')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 1, deleteId: 1, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBeNull()
    expect(plan.selectedIndexAfterDelete).toBe(0)
  })

  it('選択中（左端）を削除 → 右へ遷移', () => {
    const tabs = [t(1, 'settings', ''), t(2, 'works', '42')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 0, deleteId: 1, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBe('/works/42')
  })

  it('選択中（右端）を削除 → 左へ遷移', () => {
    const tabs = [t(1, 'works', '11'), t(2, 'works', '22')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 1, deleteId: 2, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBe('/works/11')
  })

  it('選択中（中間）を削除 → 右優先', () => {
    const tabs = [t(1, 'works', '1'), t(2, 'works', '2'), t(3, 'works', '3')]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 1, deleteId: 2, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBe('/works/3')
  })

  it('次のタブに href があればそれを優先して遷移する', () => {
    const tabs = [
      t(1, 'settings', ''),
      { ...t(2, 'works', '42'), href: '/works/42?gamename=%E3%82%B2%E3%83%BC%E3%83%A0A' },
    ]
    const plan = computeTabDeletionPlan({ tabs, selectedIndex: 0, deleteId: 1, registry: REG })
    expect(plan.shouldDelete).toBe(true)
    expect(plan.navigateToPath).toBe('/works/42?gamename=%E3%82%B2%E3%83%BC%E3%83%A0A')
  })
})
