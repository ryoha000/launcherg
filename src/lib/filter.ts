import type { Readable, Writable } from 'svelte/store'
import type { CollectionElement } from '@/lib/types'
import { writable } from 'svelte/store'
import { toHiragana, toRomaji } from 'wanakana'

export interface Option<T> { label: string, value: T, otherLabels?: string[] }

export function collectionElementsToOptions(elements: CollectionElement[]) {
  return elements.map(v => ({
    label: v.gamename,
    value: v.id,
    otherLabels: [
      toHiragana(v.gamenameRuby),
      toRomaji(v.gamenameRuby),
      v.brandname,
      toHiragana(v.brandnameRuby),
      toRomaji(v.brandnameRuby),
    ],
  }))
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
