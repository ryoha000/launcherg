export function findDetailItemIdForStoreId(storeId: string): string | null {
  const imgs = Array.from(document.querySelectorAll('img')) as HTMLImageElement[]
  for (const img of imgs) {
    const src = img.getAttribute('src') || ''
    if (!src.includes('pics.dmm.co.jp'))
      continue
    try {
      const url = new URL(src)
      const parts = url.pathname.split('/')
      if (parts.length >= 4 && parts[3] === storeId) {
        const container = img.closest('div[id]') as HTMLElement | null
        const id = container?.getAttribute('id') || null
        if (id)
          return id
      }
    }
    catch {}
  }
  return null
}

export async function fetchPackDetailHtmlForItemId(itemId: string, timeoutMs = 12000): Promise<string> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeoutMs)
  try {
    const url = new URL(`/mylibrary/detail/?item=${encodeURIComponent(itemId)}`, location.origin)
    const res = await fetch(url.toString(), { credentials: 'include', signal: controller.signal })
    if (!res.ok)
      throw new Error(`detail fetch failed: ${res.status} ${res.statusText}`)
    return await res.text()
  }
  finally {
    clearTimeout(timer)
  }
}
