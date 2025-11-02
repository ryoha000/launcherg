import type { Descriptor } from '@/router/registry'
import type { Tab } from '@/store/tabs'
import type { TabAction } from '@/store/tabs/schema'
import { buildPath } from '@/store/tabs/schema'

export function upsertSingletonTab(tabs: Tab[], action: Extract<TabAction, { mode: 'singleton' }>): { nextTabs: Tab[], selectedIndex: number } {
  const idx = tabs.findIndex(v => v.type === action.type)
  if (idx !== -1)
    return { nextTabs: tabs, selectedIndex: idx }
  const newTab: Tab = {
    id: Date.now(),
    type: action.type,
    workId: '',
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
  const keyStr = String(action.key)
  const idx = tabs.findIndex(v => v.type === action.type && v.workId === keyStr)
  if (idx !== -1)
    return { nextTabs: tabs, selectedIndex: idx }
  const newTab: Tab = {
    id: Date.now(),
    type: action.type,
    workId: keyStr,
    scrollTo: 0,
    title: action.title ?? 'エラー: タイトル不明',
  }
  const nextTabs = [...tabs, newTab]
  return { nextTabs, selectedIndex: nextTabs.length - 1 }
}

export interface DeletionPlan {
  shouldDelete: boolean
  navigateToPath: string | null
  selectedIndexAfterDelete: number | null
}

/**
 * タブ削除時の挙動を純関数として決定する。
 * - すべての判断は与えられたスナップショット（tabs, selectedIndex）に基づく。
 * - 副作用（状態更新・遷移）は行わない。
 */
export function computeTabDeletionPlan(params: {
  tabs: Tab[]
  selectedIndex: number
  deleteId: number
  registry: readonly Descriptor[]
}): DeletionPlan {
  const { tabs, selectedIndex, deleteId, registry } = params

  const deleteIndex = tabs.findIndex(t => t.id === deleteId)
  if (deleteIndex < 0)
    return { shouldDelete: false, navigateToPath: null, selectedIndexAfterDelete: null }

  // 1枚のみ → 削除後は空なので home へ
  if (tabs.length === 1)
    return { shouldDelete: true, navigateToPath: '/', selectedIndexAfterDelete: null }

  // 非選択を削除 → 選択は維持（インデックス補正のみ）、遷移なし
  if (selectedIndex < 0 || selectedIndex !== deleteIndex) {
    const selectedIndexAfterDelete = selectedIndex >= 0 && deleteIndex < selectedIndex
      ? selectedIndex - 1
      : selectedIndex
    return { shouldDelete: true, navigateToPath: null, selectedIndexAfterDelete }
  }

  // 選択中を削除 → 右優先、なければ左。両方無ければ home
  const nextTab = tabs[deleteIndex + 1] ?? tabs[deleteIndex - 1] ?? null
  if (!nextTab)
    return { shouldDelete: true, navigateToPath: '/', selectedIndexAfterDelete: null }

  const descriptor = registry.find(d => d.kind === nextTab.type)
  const path = descriptor ? buildPath(descriptor, nextTab.workId) : '/'
  return { shouldDelete: true, navigateToPath: path, selectedIndexAfterDelete: null }
}
