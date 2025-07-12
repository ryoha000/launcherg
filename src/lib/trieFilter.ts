import type { Readable } from 'svelte/store'
import type { CollectionElement } from '@/lib/types'
import { writable } from 'svelte/store'
import TrieSearch from 'trie-search'
import { toHiragana, toRomaji } from 'wanakana'
import { isNotNullOrUndefined } from '@/lib/utils'

export interface Option<T> { label: string, value: T, otherLabels?: string[] }

interface KeyValue<T> {
  key: string
  value: T
}

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

export function useTrieFilter<T>(options: Readable<Option<T>[]>, getOptions: () => Option<T>[]) {
  const query = writable('')
  const filtered = writable<Option<T>[]>([...getOptions()])

  const init = () => {
    query.set('')
    filtered.set([...getOptions()])

    const optionMap = new Map<Option<T>['value'], Option<T>>()
    for (const option of getOptions()) {
      optionMap.set(option.value, option)
    }

    const trie: TrieSearch<KeyValue<T>> = new TrieSearch<KeyValue<T>>('key')
    for (const option of getOptions()) {
      trie.add({ key: option.label, value: option.value })
      if (!option.otherLabels) {
        continue
      }
      for (const otherLabel of option.otherLabels) {
        trie.add({ key: otherLabel, value: option.value })
      }
    }

    query.subscribe((_query) => {
      if (!_query) {
        return filtered.set([...getOptions()])
      }
      const searched = trie.search(_query)
      filtered.set(
        [...new Set(searched.map(v => v.value))]
          .map(v => optionMap.get(v))
          .filter(isNotNullOrUndefined),
      )
    })
  }
  init()

  options.subscribe(() => init())

  return { query, filtered }
}
