import type { Readable, Writable } from 'svelte/store'
import type { SidebarWorkItem } from '@/store/sidebarCollectionElements'
import { writable } from 'svelte/store'
import { toHiragana, toRomaji } from 'wanakana'

export interface Option<T> { label: string, value: T, otherLabels?: string[] }

export function collectionElementsToOptions(elements: SidebarWorkItem[]) {
  return elements.map((v) => {
    const otherLabels: string[] = []
    if (v.gamenameRuby) {
      otherLabels.push(toHiragana(v.gamenameRuby))
      otherLabels.push(toRomaji(v.gamenameRuby))
    }
    if (v.brandname) {
      otherLabels.push(toHiragana(v.brandname))
      otherLabels.push(toRomaji(v.brandname))
    }
    if (v.sellday) {
      otherLabels.push(v.sellday)
    }
    return {
      label: v.title,
      value: v.id,
      otherLabels,
    }
  })
}

export function useFilter<T>(query: Writable<string>, options: Readable<Option<T>[]>, getOptions: () => Option<T>[]) {
  const filtered = writable<Option<T>[]>([...getOptions()])

  const init = () => {
    const lazyQuery = writable('')
    filtered.set([...getOptions()])

    const optionMap = new Map<Option<T>['value'], Option<T>>()
    for (const option of getOptions()) {
      optionMap.set(option.value, option)
    }

    const cache: Record<string, Option<T>[]> = {}

    let lazyQueryTimer: ReturnType<typeof setTimeout> | null = null
    query.subscribe((_query) => {
      if (lazyQueryTimer) {
        clearTimeout(lazyQueryTimer)
      }
      lazyQueryTimer = setTimeout(() => {
        lazyQuery.set(_query.toLowerCase())
        lazyQueryTimer = null
      }, 200)
    })
    lazyQuery.subscribe((_query) => {
      if (!_query) {
        return filtered.set([...getOptions()])
      }
      const cached = Object.entries(cache).find(([input, _]) =>
        _query.includes(input),
      )
      const targetOptions = cached ? cached[1] : getOptions()
      const _filtered = targetOptions.filter(option =>
        [option.label, ...(option.otherLabels ?? [])].find(key =>
          key.toLowerCase().includes(_query),
        ),
      )
      cache[_query] = _filtered
      filtered.set(_filtered)
    })
  }
  init()

  options.subscribe(() => init())

  return { query, filtered }
}
