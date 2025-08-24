import type { Tab } from '@/store/tabs'
import type { TabAction } from '@/store/tabs/schema'

export function upsertSingletonTab(tabs: Tab[], action: Extract<TabAction, { mode: 'singleton' }>): { nextTabs: Tab[], selectedIndex: number } {
  const idx = tabs.findIndex(v => v.type === action.type)
  if (idx !== -1)
    return { nextTabs: tabs, selectedIndex: idx }
  const newTab: Tab = {
    id: Date.now(),
    type: action.type,
    workId: -1,
    scrollTo: 0,
    title: action.title,
  }
  const nextTabs = [...tabs, newTab]
  return { nextTabs, selectedIndex: nextTabs.length - 1 }
}

export function upsertKeyedTab(
  tabs: Tab[],
  action: Extract<TabAction, { mode: 'keyed' }>,
): { nextTabs: Tab[], selectedIndex: number } {
  const idx = tabs.findIndex(v => v.type === action.type && String(v.workId) === String(action.key))
  if (idx !== -1)
    return { nextTabs: tabs, selectedIndex: idx }
  const newTab: Tab = {
    id: Date.now(),
    type: action.type,
    workId: typeof action.key === 'string' ? Number(action.key) : (action.key as number),
    scrollTo: 0,
    title: action.title ?? 'エラー: タイトル不明',
  }
  const nextTabs = [...tabs, newTab]
  return { nextTabs, selectedIndex: nextTabs.length - 1 }
}

export function decideNextAfterDelete(tabs: Tab[], deleteIndex: number, currentIndex: number): { nextIndex: number | null } {
  const isCurrentTab = deleteIndex === currentIndex
  const isRightestTab = deleteIndex === tabs.length - 1
  if (!isCurrentTab) {
    const isDeletePrevTab = deleteIndex < currentIndex
    return { nextIndex: isDeletePrevTab ? currentIndex - 1 : currentIndex }
  }
  const newIndex = isRightestTab ? currentIndex - 1 : currentIndex
  return { nextIndex: tabs.length === 0 ? null : newIndex }
}
