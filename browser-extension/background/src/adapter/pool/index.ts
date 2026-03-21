const DEFAULT_DEBOUNCE_MS = 30_000

export function createSyncPool<T>(options?: { debounceMs?: number }) {
  const debounceMs = options?.debounceMs ?? DEFAULT_DEBOUNCE_MS

  let toSync: T[] = []
  let lastAppendedAt = Date.now()
  let isSyncing = false

  const add = (item: T) => {
    toSync.push(item)
    lastAppendedAt = Date.now()
  }

  const sync = async (callback: (items: T[]) => Promise<void>) => {
    const now = Date.now()
    if (now - lastAppendedAt < debounceMs || isSyncing)
      return
    isSyncing = true
    await callback(toSync)
    toSync = []
    isSyncing = false
  }

  return { add, sync }
}
