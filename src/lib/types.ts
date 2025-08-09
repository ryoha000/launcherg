export interface Work {
  id: number
  erogamescapeId?: number | null
  name: string
  brandId: number
  brandName: string
  officialHomePage: string
  sellday: string
  imgUrl: string
  statistics: Statistics
  creators: Creators
  musics: string[]
}

export interface Statistics {
  median: number
  average: number
  count: number
  playTime: string
}

export interface Creators {
  illustrators: Creator[]
  writers: Creator[]
  voiceActors: VoiceActor[]
}

export interface Creator {
  id: number
  name: string
}

export const VoiceActorImportance = {
  Main: 0,
  Sub: 1,
  Mob: 2,
} as const

export type VoiceActor = {
  role: string
  importance: (typeof VoiceActorImportance)[keyof typeof VoiceActorImportance]
} & Creator

export interface Collection {
  id: number
  name: string
}

export interface CollectionElement {
  id: number // 内部ID（CollectionElementの自前採番ID）
  erogamescapeId?: number | null
  gamename: string
  gamenameRuby: string
  brandname: string
  brandnameRuby: string
  sellday: string
  isNukige: boolean
  installAt: string | null
  lastPlayAt: string | null
  likeAt: string | null
  registeredAt: string
  exePath: string
  lnkPath: string
  icon: string
  thumbnail: string
  thumbnailWidth: number | null
  thumbnailHeight: number | null
  dlStore?: DLStoreInfo | null
  installStatus: GameInstallStatus
  canPlay: boolean
  canInstall: boolean
}

export interface DLStoreInfo {
  id: number
  collectionElementId: number
  storeId: string
  storeType: 'DMM' | 'DLSite'
  storeName: string
  purchaseUrl: string
  isOwned: boolean
  purchaseDate: string | null
  createdAt: string
  updatedAt: string
}

export type GameInstallStatus = 'installed' | 'owned-not-installed' | 'not-owned'

export interface CollectionElementsWithLabel {
  label: string
  elements: CollectionElement[]
}

export type SeiyaDataPair = [string, string]

export interface CollectionElementDetail {
  collectionElementId: number
  gamename: string
  gamenameRuby: string
  brandname: string
  brandnameRuby: string
  sellday: string
  isNukige: boolean
}

export interface AllGameCacheOne {
  id: number
  gamename: string
  thumbnailUrl: string
}
