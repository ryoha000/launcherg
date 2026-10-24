// Run manually in the DLsite Play DevTools Console.
// Read-only IndexedDB inspection. No network requests or raw record values.
(async () => {
  const shape = (value, depth = 0) => {
    if (value === null)
      return 'null'
    if (typeof value !== 'object')
      return typeof value
    if (value instanceof Blob)
      return 'Blob'
    if (value instanceof ArrayBuffer || ArrayBuffer.isView(value))
      return 'binary'
    if (depth >= 5)
      return Array.isArray(value) ? 'array' : 'object'
    if (Array.isArray(value))
      return { length: value.length, item: value.length ? shape(value[0], depth + 1) : 'empty' }
    return Object.fromEntries(Object.entries(value).slice(0, 80).map(([key, child]) => [key, shape(child, depth + 1)]))
  }
  const report = []
  for (const info of await indexedDB.databases()) {
    if (!info.name)
      continue
    const db = await new Promise((resolve, reject) => {
      const request = indexedDB.open(info.name, info.version)
      request.onupgradeneeded = () => request.transaction.abort() // Never create or upgrade a DB.
      request.onsuccess = () => resolve(request.result)
      request.onerror = () => reject(request.error)
    })
    const entry = { name: db.name, version: db.version, stores: [] }
    try {
      for (const name of db.objectStoreNames) {
        const result = await new Promise((resolve, reject) => {
          const tx = db.transaction(name, 'readonly')
          const store = tx.objectStore(name)
          const summary = { name, keyPath: store.keyPath, indexes: [...store.indexNames], count: 0, scanned: 0 }
          const count = store.count()
          count.onsuccess = () => { summary.count = count.result }
          const cursor = store.openCursor()
          cursor.onsuccess = () => {
            const current = cursor.result
            if (!current)
              return
            summary.scanned++
            summary.first ??= shape(current.value)
            if (current.value?.file_type === 'EXE')
              summary.game = shape(current.value)
            if (!summary.game && summary.scanned < 500)
              current.continue()
          }
          tx.oncomplete = () => resolve(summary)
          tx.onabort = () => reject(tx.error)
        })
        entry.stores.push(result)
      }
    }
    finally {
      db.close()
    }
    report.push(entry)
  }
  const output = JSON.stringify(report, null, 2)
  console.log(output)
  if (typeof copy === 'function')
    copy(output)
  return output
})().catch(error => console.error('IndexedDB inspection failed:', error.name))
