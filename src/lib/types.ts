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

// DLStoreInfo 廃止

export interface DmmInfo {
  id: number
  workId: string
  category: string
  subcategory: string
}

export interface DlsiteInfo {
  id: number
  workId: string
  category: string
}

export type SeiyaDataPair = [string, string]

export interface AllGameCacheOne {
  id: number
  gamename: string
  thumbnailUrl: string
}
