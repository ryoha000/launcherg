import type { Attribute } from '@/components/Sidebar/searchAttributes'
import type { SortOrder } from '@/components/Sidebar/sort'
import type { Option } from '@/lib/trieFilter'
import type { SidebarWorkItemsWithLabel } from '@/store/sidebarCollectionElements'
import {

  FILTER_BY_ATTRIBUTE,
} from '@/components/Sidebar/searchAttributes'
import { sort } from '@/components/Sidebar/sort'
import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

export function search(filteredOption: Option<string>[], attributes: Attribute[], order: SortOrder): SidebarWorkItemsWithLabel[] {
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
