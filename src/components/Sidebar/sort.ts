import type { Option } from "@/lib/trieFilter";
import type {
  CollectionElement,
  CollectionElementsWithLabel,
} from "@/lib/types";
import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

export type SortOrder =
  `${(typeof SORT_ORDER_TYPES)[keyof typeof SORT_ORDER_TYPES]}-${(typeof SORT_ORDER_BY)[keyof typeof SORT_ORDER_BY]}`;

export const SORT_LABELS: { [key in SortOrder]: string } = {
  "gamename-asc": "タイトルで並び替え(昇順)",
  "gamename-desc": "タイトルで並び替え(降順)",
  "sellyear-asc": "発売年で並び替え(昇順)",
  "sellyear-desc": "発売年で並び替え",
  "brandname-asc": "ブランド名で並び替え(昇順)",
  "brandname-desc": "ブランド名で並び替え(降順)",
} as const;

export const SORT_ORDER_TYPES = {
  GAMENAME: "gamename",
  SELLYEAR: "sellyear",
  BRANDNAME: "brandname",
} as const;
export const SORT_ORDER_BY = {
  ASC: "asc",
  DESC: "desc",
} as const;

export const filterAndSort = (
  filteredOption: Option<number>[],
  order: SortOrder
): CollectionElementsWithLabel[] => {
  const filteredElements = sidebarCollectionElements
    .value()
    .filter(
      (element) =>
        filteredOption.findIndex((option) => option.value === element.id) !== -1
    );
  const isGamename = order.includes(SORT_ORDER_TYPES.GAMENAME);
  const isSellyear = order.includes(SORT_ORDER_TYPES.SELLYEAR);
  const isBrandname = order.includes(SORT_ORDER_TYPES.BRANDNAME);
  const isAsc = order.includes(SORT_ORDER_BY.ASC);
  const multiplyer = isAsc ? 1 : -1;

  if (isGamename) {
    return sortByGamename(filteredElements, multiplyer);
  }
  if (isSellyear) {
    return sortBySellyear(filteredElements, multiplyer);
  }
  if (isBrandname) {
    return sortByBrandname(filteredElements, multiplyer);
  }
  return [
    {
      label: "single label",
      elements: filteredElements,
    },
  ];
};

const sortByGamename = (elements: CollectionElement[], multiplyer: number) => [
  {
    label: "single label",
    elements: [...elements].sort((a, b) =>
      createCompareString(multiplyer)(a.gamenameRuby, b.gamenameRuby)
    ),
  },
];

const createCompareString = (multiplyer: number) => (a: string, b: string) => {
  return a.localeCompare(b, "ja") * multiplyer;
};

const sortBySellyear = (elements: CollectionElement[], multiplyer: number) =>
  elements
    .reduce((acc, cur) => {
      const year = cur.sellday.split("-")[0];
      const index = acc.findIndex((v) => v.label === year);
      if (index !== -1) {
        acc[index].elements.push(cur);
      } else {
        acc.push({ label: year, elements: [cur] });
      }
      return acc;
    }, [] as CollectionElementsWithLabel[])
    .sort((a, b) => createCompareDay(multiplyer)(a.label, b.label))
    .map((v) => ({
      ...v,
      elements: v.elements.sort((a, b) =>
        createCompareDay(multiplyer)(a.sellday, b.sellday)
      ),
    }));

const sortByBrandname = (elements: CollectionElement[], multiplyer: number) =>
  elements
    .reduce((acc, cur) => {
      const brandname = cur.brandname;
      const index = acc.findIndex((v) => v.label === brandname);
      if (index !== -1) {
        acc[index].elements.push(cur);
      } else {
        acc.push({ label: brandname, elements: [cur] });
      }
      return acc;
    }, [] as CollectionElementsWithLabel[])
    .sort((a, b) =>
      createCompareString(multiplyer)(
        a.elements[0].brandnameRuby,
        b.elements[0].brandnameRuby
      )
    )
    .map((v) => ({
      ...v,
      elements: v.elements.sort((a, b) =>
        createCompareDay(1)(a.sellday, b.sellday)
      ),
    }));

const createCompareDay = (multiplyer: number) => (a: string, b: string) => {
  const dateA = new Date(a);
  const dateB = new Date(b);
  return (dateA.getTime() - dateB.getTime()) * multiplyer;
};
