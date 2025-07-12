import type { Attribute } from '@/components/Sidebar/searchAttributes'
import type { SortOrder } from '@/components/Sidebar/sort'
import type { Option } from '@/lib/trieFilter'
import type { CollectionElementsWithLabel } from '@/lib/types'
import {

  FILTER_BY_ATTRIBUTE,
} from '@/components/Sidebar/searchAttributes'
import { sort } from '@/components/Sidebar/sort'
import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

export function search(filteredOption: Option<number>[], attributes: Attribute[], order: SortOrder): CollectionElementsWithLabel[] {
  const filteredElements = sidebarCollectionElements
    .value()
    .filter(
      element =>
        filteredOption.findIndex(option => option.value === element.id) !== -1,
    )

  const filtered = attributes.reduce(
    (acc, cur) => (cur.enabled ? FILTER_BY_ATTRIBUTE[cur.key](acc) : acc),
    filteredElements,
  )

  return sort(filtered, order)
}
