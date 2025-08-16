import type { DlsiteGame, DmmGame, EgsInfo } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessage, NativeResponse } from '@launcherg/shared/proto/native_messaging'

export interface NativeMessenger {
  send: (message: NativeMessage) => Promise<NativeResponse | null>
}

export interface EgsResolver {
  resolveForDmm: (storeId: string, category: string, subcategory: string) => Promise<EgsInfo | null>
  resolveForDlsite: (storeId: string, category: string) => Promise<EgsInfo | null>
  resolveForDmmBulk: (items: Array<{ storeId: string, category: string, subcategory: string }>) => Promise<Array<EgsInfo | null>>
  resolveForDlsiteBulk: (items: Array<{ storeId: string, category: string }>) => Promise<Array<EgsInfo | null>>
}

export interface Aggregation {
  record: (count: number) => Promise<void>
}

export interface IdGenerator {
  generate: () => string
}

export interface BrowserTab {
  id?: number
  url?: string
}

export interface Browser {
  alarms: {
    create: (name: string, options: { when?: number, delayInMinutes?: number, periodInMinutes?: number }) => Promise<void> | void
  }
  notifications: {
    create: (options: { type: 'basic', iconUrl: string, title: string, message: string }) => Promise<void>
  }
  runtime: {
    getURL: (path: string) => string
  }
  storage: {
    get: (keys: string[]) => Promise<Record<string, any>>
    set: (items: Record<string, any>) => Promise<void>
  }
  tabs: {
    query: (queryInfo: chrome.tabs.QueryInfo) => Promise<BrowserTab[]>
    sendMessage: (tabId: number, message: unknown) => Promise<void>
  }
  scripting: {
    executeScript: (tabId: number, files: string[]) => Promise<void>
  }
}

export interface SyncPool<T> {
  add: (items: T) => void
  sync: (callback: (items: T[]) => Promise<void>) => Promise<void>
}

export interface HandlerContext {
  extensionId: string
  nativeHostName: string
  nativeMessenger: NativeMessenger
  egsResolver: EgsResolver
  aggregation: Aggregation
  idGenerator: IdGenerator
  browser: Browser
  syncPool: SyncPool<{ type: 'dmm', games: DmmGame[] } | { type: 'dlsite', games: DlsiteGame[] }>
}

export {}
