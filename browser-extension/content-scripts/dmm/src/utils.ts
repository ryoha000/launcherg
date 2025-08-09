// URL から cid を抽出
export function extractCidFromUrl(href: string): string | null {
  if (!href)
    return null
  const m = href.match(/[?&]cid=([^&]+)/)
  return m ? decodeURIComponent(m[1]) : null
}
