// string キー専用の KV キャッシュ
// - V: 保管する値の型
export function createCache<V>(namespace: string) {
  const KEY_PREFIX = `launcherg:${namespace}:`
  return {
    async get(key: string): Promise<V | null> {
      const storageKey = KEY_PREFIX + key
      return new Promise((resolve) => {
        chrome.storage.local.get([storageKey], (items) => {
          const raw = items[storageKey] as unknown
          resolve(raw === undefined ? null : (raw as V))
        })
      })
    },
    async set(key: string, value: V): Promise<void> {
      const storageKey = KEY_PREFIX + key
      return new Promise((resolve) => {
        chrome.storage.local.set({ [storageKey]: value }, () => resolve())
      })
    },
  }
}
