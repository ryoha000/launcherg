import type { EgsInfo } from '@launcherg/shared/proto/extension_internal'

export interface NativeMessenger {
  send: (message: any) => Promise<any | null>
}

export interface EgsResolver {
  resolveForDmm: (storeId: string, category: string, subcategory: string) => Promise<EgsInfo | null>
  resolveForDlsite: (storeId: string, category: string) => Promise<EgsInfo | null>
  resolveForDmmBulk?: (items: Array<{ storeId: string, category: string, subcategory: string }>) => Promise<Array<EgsInfo | null>>
  resolveForDlsiteBulk?: (items: Array<{ storeId: string, category: string }>) => Promise<Array<EgsInfo | null>>
}

export interface Aggregation {
  record: (count: number) => Promise<void>
}

export interface IdGenerator {
  generate: () => string
}

export interface HandlerContext {
  extensionId: string
  nativeHostName: string
  nativeMessenger: NativeMessenger
  egsResolver: EgsResolver
  aggregation: Aggregation
  idGenerator: IdGenerator
}

export {}
