import { commandGetWorkDetailsAll, commandUpdateWorkLike } from '@/lib/command'
import { createWritable } from '@/lib/utils'

export interface SidebarWorkItem {
  id: number
  title: string
  thumbnail?: { path: string, width?: number, height?: number }
  likeAt?: string | null
  installAt?: string | null
  lastPlayAt?: string | null
  registeredAt?: string | null
  // EGS derived fields for sort/filter
  gamenameRuby?: string
  brandname?: string
  brandnameRuby?: string
  sellday?: string
  isNukige?: boolean
  hasPath?: boolean
}

export interface SidebarWorkItemsWithLabel {
  label: string
  elements: SidebarWorkItem[]
}

function createSidebarCollectionElements() {
  const [{ subscribe, update, set }, value] = createWritable<SidebarWorkItem[]>([])

  const refetch = async () => {
    const rows = await commandGetWorkDetailsAll()
    set(
      rows.map(v => ({
        id: v.id,
        title: v.title,
        thumbnail: v.thumbnail ?? undefined,
        likeAt: v.likeAt ?? null,
        installAt: v.installAt ?? null,
        lastPlayAt: v.lastPlayAt ?? null,
        registeredAt: v.registeredAt ?? null,
        gamenameRuby: v.erogamescapeInformation?.gamenameRuby,
        brandname: v.erogamescapeInformation?.brandname,
        brandnameRuby: v.erogamescapeInformation?.brandnameRuby,
        sellday: v.erogamescapeInformation?.sellday,
        isNukige: v.erogamescapeInformation?.isNukige,
        hasPath: !!v.latestDownloadPath?.downloadPath,
      })),
    )
  }

  const updateLike = async (workId: number, isLike: boolean) => {
    await commandUpdateWorkLike(workId, isLike)
    const now = new Date()
    const likeAt = isLike
      ? `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`
      : null
    update(elements => elements.map(v => (v.id === workId ? { ...v, likeAt } : { ...v })))
  }

  const [shown] = createWritable<SidebarWorkItemsWithLabel[]>([])

  return {
    subscribe,
    value,
    refetch,
    updateLike,
    shown,
  }
}

export const sidebarCollectionElements = createSidebarCollectionElements()
