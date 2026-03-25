import type { EgsInfo } from '@launcherg/shared'
import type { NativeMessageTs, NativeResponseTs } from '@launcherg/shared/typeshare/native-messaging'

export interface NativeMessenger {
  sendJson: (message: NativeMessageTs) => Promise<NativeResponseTs | null>
}

export interface EgsResolver {
  resolveForDmm: (storeId: string, category: string, subcategory: string) => Promise<EgsInfo | null>
  resolveForDlsite: (storeId: string, category: string) => Promise<EgsInfo | null>
  resolveForDmmBulk: (items: Array<{ storeId: string, category: string, subcategory: string }>) => Promise<Array<EgsInfo | null>>
  resolveForDlsiteBulk: (items: Array<{ storeId: string, category: string }>) => Promise<Array<EgsInfo | null>>
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
    executeScript: (tabId: number, files: string[], world?: chrome.scripting.ExecutionWorld) => Promise<void>
  }
}

export interface SyncCoordinator {
  runExclusive: <T>(callback: () => Promise<T>) => Promise<T>
}

export interface HandlerContext {
  extensionId: string
  nativeHostName: string
  nativeMessenger: NativeMessenger
  egsResolver: EgsResolver
  idGenerator: IdGenerator
  browser: Browser
  syncCoordinator: SyncCoordinator
}

export {}
