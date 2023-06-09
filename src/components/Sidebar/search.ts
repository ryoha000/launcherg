import { type SortOrder, sort } from "@/components/Sidebar/sort";
import type { CollectionElementsWithLabel } from "@/lib/types";
import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
import type { Option } from "@/lib/trieFilter";
import {
  FILTER_BY_ATTRIBUTE,
  type Attribute,
} from "@/components/Sidebar/searchAttributes";

export const search = (
  filteredOption: Option<number>[],
  attributes: Attribute[],
  order: SortOrder
): CollectionElementsWithLabel[] => {
  const filteredElements = sidebarCollectionElements
    .value()
    .filter(
      (element) =>
        filteredOption.findIndex((option) => option.value === element.id) !== -1
    );

  const filtered = attributes.reduce(
    (acc, cur) => (cur.enabled ? FILTER_BY_ATTRIBUTE[cur.key](acc) : acc),
    filteredElements
  );

  return sort(filtered, order);
};
