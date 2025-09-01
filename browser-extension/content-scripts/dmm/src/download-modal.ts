export function collectDownloadLinks(root: Document | HTMLElement): HTMLAnchorElement[] {
  const scope = (root instanceof Document)
    ? (root.getElementById('js-detail') ?? root)
    : root

  // 仕様に依存しない抽出: href のパターンで判定
  // - 絶対/相対どちらもあり得るため、文字列包含で確認
  // - 転送種別 transfer_type=download を必須とする
  const allAnchors = Array.from(scope.querySelectorAll('a')) as HTMLAnchorElement[]
  const candidates = allAnchors.filter((a) => {
    const href = a.getAttribute('href') || ''
    if (!href)
      return false
    return href.includes('/mylibrary/proxy') && href.includes('transfer_type=download')
  })

  // 重複除去（同じ href のみ）。順序は出現順を維持
  const seen = new Set<string>()
  const unique: HTMLAnchorElement[] = []
  for (const a of candidates) {
    const href = a.getAttribute('href') || ''
    if (seen.has(href))
      continue
    seen.add(href)
    unique.push(a)
  }

  return unique
}
