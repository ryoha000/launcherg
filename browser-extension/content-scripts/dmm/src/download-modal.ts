export function collectDownloadLinks(root: Document | HTMLElement, storeId: string): HTMLAnchorElement[] {
  const scope = (root instanceof Document)
    ? (root.getElementById('js-detail') ?? root)
    : root

  // 抽出対象の a 要素集合を決定
  // パック詳細(#detail_mylibrary_pack)が存在する場合は、可視な子詳細(div[id^="detail_"])の中だけを探索
  const packDetail = scope.querySelector('#detail_mylibrary_pack') as HTMLElement | null
  let allAnchors: HTMLAnchorElement[]
  if (packDetail) {
    const detailNodes = Array.from(packDetail.querySelectorAll('div[id^="detail_"]')) as HTMLElement[]
    if (detailNodes.length > 0) {
      // 表示状態に依らず、全子詳細から探索
      allAnchors = detailNodes.flatMap(d => Array.from(d.querySelectorAll('a')) as HTMLAnchorElement[])
    }
    else {
      // 空の packDetail は単品レイアウト。通常の全体探索にフォールバック
      allAnchors = Array.from(scope.querySelectorAll('a')) as HTMLAnchorElement[]
    }
  }
  else {
    allAnchors = Array.from(scope.querySelectorAll('a')) as HTMLAnchorElement[]
  }

  // 仕様に依存しない抽出: href のパターンで判定
  // - 絶対/相対どちらもあり得るため、文字列包含で確認
  // - 転送種別 transfer_type=download を必須とする
  const candidates = allAnchors.filter((a) => {
    const href = a.getAttribute('href') || ''
    if (!href)
      return false
    if (!(href.includes('/mylibrary/proxy') && href.includes('transfer_type=download')))
      return false
    // storeId(pid) で厳密フィルタ
    try {
      const url = new URL(href, 'https://dlsoft.dmm.co.jp')
      const pid = url.searchParams.get('pid') || ''
      return pid === storeId
    }
    catch {
      return false
    }
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
