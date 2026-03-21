import { localStorageWritable } from '@/lib/utils'

export interface Settings {
  storeMapped: {
    autoDeleteOnCheck: boolean
  }
}

const initialSettings: Settings = {
  storeMapped: {
    autoDeleteOnCheck: false,
  },
}

export const settings = localStorageWritable<Settings>('settings', initialSettings)
