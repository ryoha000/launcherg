import type { DlsiteWorkItem } from './api'

interface PurchaseCacheOptions {
  onSnapshot: (works: DlsiteWorkItem[]) => void
  onStatus: (stage: string, detail?: string) => void
}

// Observe the connection opened by the current DLsite page. Enumerating every
// Play_* database would also import caches left by other signed-in accounts.
export function observeDlsitePurchaseCache(options: PurchaseCacheOptions): () => void {
  const originalOpen = indexedDB.open.bind(indexedDB)
  const observedConnections = new WeakSet<IDBDatabase>()
  let active: { db: IDBDatabase, transaction: IDBDatabase['transaction'] } | undefined
  let timer: ReturnType<typeof setTimeout> | undefined
  let reading = false
  let pending = false

  const isLibraryPage = () => window.location.pathname === '/' || window.location.pathname.includes('/library')

  async function readSnapshot(): Promise<void> {
    if (!active || !isLibraryPage())
      return
    if (reading) {
      pending = true
      return
    }
    const selected = active
    reading = true
    options.onStatus('cache_reading')
    try {
      const works = await new Promise<DlsiteWorkItem[]>((resolve, reject) => {
        const transaction = selected.transaction('Purchase', 'readonly')
        const store = transaction.objectStore('Purchase')
        if (store.keyPath !== 'workno') {
          transaction.abort()
          reject(new Error('購入キャッシュの形式が未対応です'))
          return
        }
        const works: DlsiteWorkItem[] = []
        const request = store.openCursor()
        request.onsuccess = () => {
          const cursor = request.result
          if (!cursor)
            return
          const value = cursor.value as DlsiteWorkItem
          if (value && typeof value === 'object') {
            // Only copy fields needed by the existing sync pipeline.
            works.push({ workno: value.workno, site_id: value.site_id, file_type: value.file_type, name: value.name, images: value.images, work_files: value.work_files })
          }
          cursor.continue()
        }
        transaction.oncomplete = () => resolve(works)
        transaction.onabort = () => reject(transaction.error || new Error('購入キャッシュを読み取れませんでした'))
      })
      if (active === selected && isLibraryPage()) {
        options.onStatus('cache_read')
        options.onSnapshot(works)
      }
    }
    catch (error) {
      options.onStatus('cache_failed', error instanceof Error ? error.message : '購入キャッシュを読み取れませんでした')
    }
    finally {
      reading = false
      if (pending) {
        pending = false
        schedule()
      }
    }
  }

  function schedule(): void {
    if (timer)
      clearTimeout(timer)
    timer = setTimeout(() => {
      timer = undefined
      void readSnapshot()
    }, 150)
  }

  indexedDB.open = (name: string, version?: number): IDBOpenDBRequest => {
    const request = version === undefined ? originalOpen(name) : originalOpen(name, version)
    if (!name.startsWith('Play_'))
      return request
    request.addEventListener('success', () => {
      const db = request.result
      if (!db.objectStoreNames.contains('Purchase') || observedConnections.has(db))
        return
      observedConnections.add(db)
      const originalTransaction = db.transaction.bind(db)
      active = { db, transaction: originalTransaction }
      options.onStatus('cache_opened')
      // Follow committed purchases when the site populates or refreshes its
      // cache after opening it. The extension itself only starts readonly work.
      db.transaction = (...args: Parameters<IDBDatabase['transaction']>): IDBTransaction => {
        const transaction = originalTransaction(...args)
        if (transaction.mode === 'readwrite' && transaction.objectStoreNames.contains('Purchase')) {
          transaction.addEventListener('complete', () => {
            if (active?.db === db)
              schedule()
          })
        }
        return transaction
      }
      schedule()
    })
    return request
  }

  return schedule
}
