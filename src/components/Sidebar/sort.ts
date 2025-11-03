import type { SidebarWorkItem, SidebarWorkItemsWithLabel } from '@/store/sidebarWorks'

export type SortOrder
  = `${(typeof SORT_ORDER_TYPES)[keyof typeof SORT_ORDER_TYPES]}-${(typeof SORT_ORDER_BY)[keyof typeof SORT_ORDER_BY]}`

export const SORT_LABELS: { [key in SortOrder]: string } = {
  'gamename-asc': 'タイトル(昇順)',
  'gamename-desc': 'タイトル(降順)',
  'sellyear-asc': '発売年(昇順)',
  'sellyear-desc': '発売年',
  'brandname-asc': 'ブランド名(昇順)',
  'brandname-desc': 'ブランド名(降順)',
  'install-asc': 'インストールした年(昇順)',
  'install-desc': 'インストールした年',
  'last_play-asc': '最後に起動した年(昇順)',
  'last_play-desc': '最後に起動した年',
  'registered-asc': '登録した年(昇順)',
  'registered-desc': '登録した年',
} as const

export const SORT_ORDER_TYPES = {
  GAMENAME: 'gamename',
  SELLYEAR: 'sellyear',
  BRANDNAME: 'brandname',
  INSTALL: 'install',
  LAST_PLAY: 'last_play',
  REGISTERED: 'registered',
} as const
export const SORT_ORDER_BY = {
  ASC: 'asc',
  DESC: 'desc',
} as const

const NULL_DATE = '不明'

const sortByInstall = createSortByNullableDate('installAt')
const sortByLastPlay = createSortByNullableDate('lastPlayAt')
const sortByRegistered = createSortByNullableDate('registeredAt')

export function sort(filteredElements: SidebarWorkItem[], order: SortOrder): SidebarWorkItemsWithLabel[] {
  const isGamename = order.includes(SORT_ORDER_TYPES.GAMENAME)
  const isSellyear = order.includes(SORT_ORDER_TYPES.SELLYEAR)
  const isBrandname = order.includes(SORT_ORDER_TYPES.BRANDNAME)
  const isInstall = order.includes(SORT_ORDER_TYPES.INSTALL)
  const isLastPlay = order.includes(SORT_ORDER_TYPES.LAST_PLAY)
  const isRegistered = order.includes(SORT_ORDER_TYPES.REGISTERED)
  const isAsc = order.includes(SORT_ORDER_BY.ASC)
  const multiplyer = isAsc ? 1 : -1

  if (isGamename) {
    return sortByGamename(filteredElements, multiplyer)
  }
  if (isSellyear) {
    return sortBySellyear(filteredElements, multiplyer)
  }
  if (isBrandname) {
    return sortByBrandname(filteredElements, multiplyer)
  }
  if (isInstall) {
    return sortByInstall(filteredElements, multiplyer)
  }
  if (isLastPlay) {
    return sortByLastPlay(filteredElements, multiplyer)
  }
  if (isRegistered) {
    return sortByRegistered(filteredElements, multiplyer)
  }
  return [{ label: 'すべて', elements: filteredElements }]
}

function sortByGamename(elements: SidebarWorkItem[], multiplyer: number) {
  return [
    {
      label: 'すべて',
      elements: [...elements].sort((a, b) =>
        createCompareNameAndRuby(multiplyer, {
          name: 'title',
          ruby: 'gamenameRuby',
        })(a, b),
      ),
    },
  ]
}

type NameRubyKey
  = | { name: 'title', ruby: 'gamenameRuby' }
    | { name: 'brandname', ruby: 'brandnameRuby' }

function createCompareNameAndRuby(multiplyer: number, prop: NameRubyKey) {
  return (a: SidebarWorkItem, b: SidebarWorkItem) => {
    const aName = a[prop.name] ?? ''
    const bName = b[prop.name] ?? ''
    const aCode = aName.charCodeAt(0)
    const bCode = bName.charCodeAt(0)

    if (aCode < 128 && bCode < 128) {
      // ASCII characters
      return aName.localeCompare(bName) * multiplyer
    }
    else if (aCode < 128) {
      // a is ASCII, b is non-ASCII
      return -1 * multiplyer
    }
    else if (bCode < 128) {
      // a is non-ASCII, b is ASCII
      return 1 * multiplyer
    }
    else {
      // both non-ASCII
      const aRuby = a[prop.ruby] ?? aName
      const bRuby = b[prop.ruby] ?? bName
      return aRuby.localeCompare(bRuby, 'ja') * multiplyer
    }
  }
}

function sortBySellyear(elements: SidebarWorkItem[], multiplyer: number) {
  return elements
    .reduce((acc, cur) => {
      const year = cur.sellday ? cur.sellday.split('-')[0] : NULL_DATE
      const index = acc.findIndex(v => v.label === year)
      if (index !== -1) {
        acc[index].elements.push(cur)
      }
      else {
        acc.push({ label: year, elements: [cur] })
      }
      return acc
    }, [] as SidebarWorkItemsWithLabel[])
    .sort((a, b) =>
      a.label === NULL_DATE
        ? 1
        : b.label === NULL_DATE
          ? -1
          : createCompareDay(multiplyer)(a.label, b.label),
    )
    .map(v => ({
      ...v,
      elements: v.elements.sort((a, b) =>
        createCompareNullableDay(multiplyer)(a.sellday ?? null, b.sellday ?? null),
      ),
    }))
}

function sortByBrandname(elements: SidebarWorkItem[], multiplyer: number) {
  return elements
    .reduce((acc, cur) => {
      const brandname = cur.brandname ?? NULL_DATE
      const index = acc.findIndex(v => v.label === brandname)
      if (index !== -1) {
        acc[index].elements.push(cur)
      }
      else {
        acc.push({ label: brandname, elements: [cur] })
      }
      return acc
    }, [] as SidebarWorkItemsWithLabel[])
    .sort((a, b) =>
      a.label === NULL_DATE
        ? 1
        : b.label === NULL_DATE
          ? -1
          : createCompareNameAndRuby(multiplyer, { name: 'brandname', ruby: 'brandnameRuby' })(a.elements[0], b.elements[0]),
    )
    .map(v => ({
      ...v,
      elements: v.elements.sort((a, b) =>
        createCompareNullableDay(1)(a.sellday ?? null, b.sellday ?? null),
      ),
    }))
}

function createCompareDay(multiplyer: number) {
  return (a: string, b: string) => {
    const dateA = new Date(a)
    const dateB = new Date(b)
    return (dateA.getTime() - dateB.getTime()) * multiplyer
  }
}

function createCompareNullableDay(multiplyer: number) {
  return (a: string | null, b: string | null) => {
    return (
      ((a ? new Date(a).getTime() : 0) - (b ? new Date(b).getTime() : 0))
      * multiplyer
    )
  }
}

function createSortByNullableDate(key: 'installAt' | 'lastPlayAt' | 'registeredAt') {
  return (elements: SidebarWorkItem[], multiplyer: number) =>
    elements
      .reduce((acc, cur) => {
        const value = cur[key]
        const year = value ? `${new Date(value).getFullYear()}` : NULL_DATE
        const index = acc.findIndex(v => v.label === year)
        if (index !== -1) {
          acc[index].elements.push(cur)
        }
        else {
          acc.push({ label: year, elements: [cur] })
        }
        return acc
      }, [] as SidebarWorkItemsWithLabel[])
      .sort((a, b) =>
        a.label === NULL_DATE
          ? 1
          : b.label === NULL_DATE
            ? -1
            : createCompareDay(multiplyer)(a.label, b.label),
      )
      .map(v => ({
        ...v,
        elements: v.elements.sort((a, b) =>
          createCompareNullableDay(multiplyer)(a[key] ?? null, b[key] ?? null),
        ),
      }))
}
