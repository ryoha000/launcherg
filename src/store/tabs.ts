import type { Hook } from '@mateothegreat/svelte5-router'
import { goto } from '@mateothegreat/svelte5-router'
import { createLocalStorageWritable } from '@/lib/utils'
import { ROUTE_REGISTRY } from '@/router/const'
import { buildPath, getTabActionFromLocation } from '@/store/tabs/schema'
import { decideNextAfterDelete, upsertKeyedTab, upsertSingletonTab } from '@/store/tabs/updaters'

export interface Tab {
  id: number
  workId: number
  type: string
  scrollTo: number
  title: string
}

function createTabs() {
  const [tabs, getTabs] = createLocalStorageWritable<Tab[]>('tabs', [])

  const [selected, getSelected] = createLocalStorageWritable('tab-selected', 0)

  const routeLoaded: Hook = (event) => {
    const path = event.result.path.original
    const queryParams = typeof event.result.querystring.params === 'object'
      ? (event.result.querystring.params as Record<string, unknown>)
      : undefined

    const action = getTabActionFromLocation(ROUTE_REGISTRY, {
      path,
      pathParams: event.result.path.params as Record<string, unknown> | undefined,
      queryParams,
    })

    switch (action.mode) {
      case 'none': {
        selected.set(-1)
        return true
      }
      case 'singleton': {
        const { nextTabs, selectedIndex } = upsertSingletonTab(getTabs(), action)
        tabs.set(nextTabs)
        selected.set(selectedIndex)
        return true
      }
      case 'keyed': {
        const { nextTabs, selectedIndex } = upsertKeyedTab(getTabs(), action)
        tabs.set(nextTabs)
        selected.set(selectedIndex)
        return true
      }
      default: {
        const _exhaustive: never = action
        return _exhaustive
      }
    }
  }
  const deleteTab = (id: number) => {
    const deleteIndex = getTabs().findIndex(v => v.id === id)
    const currentIndex = getSelected()

    const isRightestTab = deleteIndex === getTabs().length - 1

    tabs.update((v) => {
      const newTabs = v.filter(tab => tab.id !== id)
      if (newTabs.length === 0) {
        goto('/')
      }
      return newTabs
    })

    if (isRightestTab && getTabs().length === 0) {
      return
    }

    const { nextIndex } = decideNextAfterDelete(getTabs(), deleteIndex, currentIndex)
    if (nextIndex === null)
      return
    const nextTab = getTabs()[nextIndex]
    const descriptor = ROUTE_REGISTRY.find(d => d.kind === nextTab.type)
    if (descriptor) {
      goto(buildPath(descriptor, nextTab.workId))
    }
    else {
      goto('/')
    }
  }
  const initialize = () => {
    const _tabs = getTabs()
    const index = getSelected()
    if (_tabs.length - 1 < index) {
      console.error('_tabs.length - 1 < index', {
        tabs: getTabs(),
        selected: getSelected(),
      })
      selected.set(-1)
      goto('/')
      return
    }
    if (index < 0) {
      goto('/')
      return
    }
    const tab = _tabs[index]
    const descriptor = ROUTE_REGISTRY.find(d => d.kind === tab.type)
    if (descriptor) {
      goto(buildPath(descriptor, tab.workId))
    }
    else {
      goto('/')
    }
  }
  const getSelectedTab = () => getTabs()[getSelected()]
  return {
    tabs,
    selected: {
      subscribe: selected.subscribe,
    },
    getSelectedTab,
    routeLoaded,
    deleteTab,
    initialize,
  }
}

export const {
  tabs,
  selected,
  getSelectedTab,
  routeLoaded,
  deleteTab,
  initialize,
} = createTabs()
