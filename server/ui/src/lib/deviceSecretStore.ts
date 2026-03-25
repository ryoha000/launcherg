const DB_NAME = 'launcherg-remote-share'
const DB_VERSION = 1
const STORE_NAME = 'device-secrets'

type StoredDeviceSecret = {
  deviceId: string
  deviceSecret: string
}

let dbPromise: Promise<IDBDatabase> | null = null

function openDatabase(): Promise<IDBDatabase> {
  if (dbPromise) {
    return dbPromise
  }

  dbPromise = new Promise((resolve, reject) => {
    if (!('indexedDB' in window)) {
      reject(new Error('IndexedDB is not available'))
      return
    }

    const request = window.indexedDB.open(DB_NAME, DB_VERSION)

    request.onupgradeneeded = () => {
      const database = request.result
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        database.createObjectStore(STORE_NAME, { keyPath: 'deviceId' })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error ?? new Error('Failed to open IndexedDB'))
  })

  return dbPromise
}

async function withStore<T>(
  mode: IDBTransactionMode,
  handler: (store: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  const database = await openDatabase()

  return await new Promise<T>((resolve, reject) => {
    const transaction = database.transaction(STORE_NAME, mode)
    const store = transaction.objectStore(STORE_NAME)
    const request = handler(store)

    request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'))

    if (mode === 'readonly') {
      request.onsuccess = () => resolve(request.result)
      return
    }

    transaction.oncomplete = () => resolve(request.result)
    transaction.onabort = () => reject(transaction.error ?? new Error('IndexedDB transaction aborted'))
    transaction.onerror = () => reject(transaction.error ?? new Error('IndexedDB transaction failed'))
  })
}

export async function getStoredDeviceSecret(deviceId: string): Promise<string | null> {
  try {
    const record = await withStore<StoredDeviceSecret | undefined>('readonly', store => store.get(deviceId))
    return record?.deviceSecret ?? null
  }
  catch {
    return null
  }
}

export async function setStoredDeviceSecret(deviceId: string, deviceSecret: string): Promise<void> {
  await withStore('readwrite', store => store.put({ deviceId, deviceSecret }))
}

export async function deleteStoredDeviceSecret(deviceId: string): Promise<void> {
  await withStore('readwrite', store => store.delete(deviceId))
}

export async function getAllStoredDeviceIds(): Promise<string[]> {
  const records = await withStore<StoredDeviceSecret[]>('readonly', (store) => store.getAll())
  return records.map((record) => record.deviceId)
}
