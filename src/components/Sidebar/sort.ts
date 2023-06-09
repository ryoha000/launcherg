import type {
  CollectionElement,
  CollectionElementsWithLabel,
} from "@/lib/types";

export type SortOrder =
  `${(typeof SORT_ORDER_TYPES)[keyof typeof SORT_ORDER_TYPES]}-${(typeof SORT_ORDER_BY)[keyof typeof SORT_ORDER_BY]}`;

export const SORT_LABELS: { [key in SortOrder]: string } = {
  "gamename-asc": "タイトルで並び替え(昇順)",
  "gamename-desc": "タイトルで並び替え(降順)",
  "sellyear-asc": "発売年で並び替え(昇順)",
  "sellyear-desc": "発売年で並び替え",
  "brandname-asc": "ブランド名で並び替え(昇順)",
  "brandname-desc": "ブランド名で並び替え(降順)",
  "install-asc": "インストールした年で並び替え(昇順)",
  "install-desc": "インストールした年で並び替え",
  "last_play-asc": "最後に起動した年で並び替え(昇順)",
  "last_play-desc": "最後に起動した年で並び替え",
} as const;

export const SORT_ORDER_TYPES = {
  GAMENAME: "gamename",
  SELLYEAR: "sellyear",
  BRANDNAME: "brandname",
  INSTALL: "install",
  LAST_PLAY: "last_play",
} as const;
export const SORT_ORDER_BY = {
  ASC: "asc",
  DESC: "desc",
} as const;

const NULL_DATE = "不明";

export const sort = (
  filteredElements: CollectionElement[],
  order: SortOrder
): CollectionElementsWithLabel[] => {
  const isGamename = order.includes(SORT_ORDER_TYPES.GAMENAME);
  const isSellyear = order.includes(SORT_ORDER_TYPES.SELLYEAR);
  const isBrandname = order.includes(SORT_ORDER_TYPES.BRANDNAME);
  const isInstall = order.includes(SORT_ORDER_TYPES.INSTALL);
  const isLastPlay = order.includes(SORT_ORDER_TYPES.LAST_PLAY);
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
  if (isInstall) {
    return sortByInstall(filteredElements, multiplyer);
  }
  if (isLastPlay) {
    return sortByLastPlay(filteredElements, multiplyer);
  }
  return [
    {
      label: "すべて",
      elements: filteredElements,
    },
  ];
};

const sortByGamename = (elements: CollectionElement[], multiplyer: number) => [
  {
    label: "すべて",
    elements: [...elements].sort((a, b) =>
      createCompareNameAndRuby(multiplyer, {
        name: "gamename",
        ruby: "gamenameRuby",
      })(a, b)
    ),
  },
];

const createCompareNameAndRuby =
  (
    multiplyer: number,
    prop:
      | { name: "gamename"; ruby: "gamenameRuby" }
      | { name: "brandname"; ruby: "brandnameRuby" }
  ) =>
  (a: CollectionElement, b: CollectionElement) => {
    const aCode = a[prop.name].charCodeAt(0);
    const bCode = b[prop.name].charCodeAt(0);

    if (aCode < 128 && bCode < 128) {
      // ASCII characters
      return a[prop.name].localeCompare(b[prop.name]) * multiplyer;
    } else if (aCode < 128) {
      // a is ASCII, b is non-ASCII
      return -1 * multiplyer;
    } else if (bCode < 128) {
      // a is non-ASCII, b is ASCII
      return 1 * multiplyer;
    } else {
      // both non-ASCII
      return a[prop.ruby].localeCompare(b[prop.ruby], "ja") * multiplyer;
    }
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
      createCompareNameAndRuby(multiplyer, {
        name: "brandname",
        ruby: "brandnameRuby",
      })(a.elements[0], b.elements[0])
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

const createSortByNullableDate =
  (key: "installAt" | "lastPlayAt") =>
  (elements: CollectionElement[], multiplyer: number) =>
    elements
      .reduce((acc, cur) => {
        const value = cur[key];
        const year = value ? value.split("-")[0] : NULL_DATE;
        const index = acc.findIndex((v) => v.label === year);
        if (index !== -1) {
          acc[index].elements.push(cur);
        } else {
          acc.push({ label: year, elements: [cur] });
        }
        return acc;
      }, [] as CollectionElementsWithLabel[])
      .sort((a, b) =>
        a.label === NULL_DATE
          ? 1
          : b.label === NULL_DATE
          ? -1
          : createCompareDay(multiplyer)(a.label, b.label)
      )
      .map((v) => ({
        ...v,
        elements: v.elements.sort((a, b) =>
          createCompareDay(multiplyer)(a.sellday, b.sellday)
        ),
      }));

const sortByInstall = createSortByNullableDate("installAt");
const sortByLastPlay = createSortByNullableDate("lastPlayAt");
