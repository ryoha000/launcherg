// 汎用ユーティリティ関数

// リクエストIDを生成する純粋関数
export function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// ページが完全に読み込まれるまで待機する関数
export function waitForPageLoad(delay: number = 1000): Promise<void> {
  return new Promise((resolve) => {
    if (document.readyState === 'complete') {
      setTimeout(resolve, delay)
    }
    else {
      window.addEventListener('load', () => {
        setTimeout(resolve, delay)
      })
    }
  })
}

// タイトル正規化（DMM / DLsite 共通）
// - 角括弧【...】や丸括弧（...）/ (...)、角括弧[...]内の付帯情報を除去
// - 連続空白を単一空白へ圧縮し、前後空白を除去
function decodeHtmlEntities(input: string): string {
  const named: Record<string, string> = {
    amp: '&',
    lt: '<',
    gt: '>',
    quot: '"',
    apos: '\'',
    nbsp: ' ',
    lsquo: '‘',
    rsquo: '’',
    ldquo: '“',
    rdquo: '”',
    ndash: '–',
    mdash: '—',
    hellip: '…',
    middot: '・',
    bull: '•',
    lpar: '(',
    rpar: ')',
    lsqb: '[',
    rsqb: ']',
    lbrack: '[',
    rbrack: ']',
    euro: '€',
    pound: '£',
    yen: '¥',
    cent: '¢',
    copy: '©',
    reg: '®',
    trade: '™',
    times: '×',
    divide: '÷',
    plusmn: '±',
    micro: 'µ',
    deg: '°',
    sup2: '²',
    sup3: '³',
    frac14: '¼',
    frac12: '½',
    frac34: '¾',
    dagger: '†',
    Dagger: '‡',
  }

  // 数値参照（10進/16進）
  const numericDecoded = input.replace(/&#(x?[0-9A-Fa-f]+);/g, (_, num: string) => {
    try {
      const codePoint = num.startsWith('x') || num.startsWith('X') ? Number.parseInt(num.slice(1), 16) : Number.parseInt(num, 10)
      if (!Number.isFinite(codePoint) || codePoint <= 0)
        return _
      return String.fromCodePoint(codePoint)
    }
    catch {
      return _
    }
  })

  // 名前付きエンティティ
  return numericDecoded.replace(/&([a-z]+);/gi, (m, name: string) => {
    return Object.prototype.hasOwnProperty.call(named, name) ? named[name] : m
  })
}

export function normalizeTitle(raw: string): string {
  return decodeHtmlEntities(raw || '')
    .replace(/【.*?】/g, '')
    .replace(/\[.*?\]/g, '')
    .replace(/（.*?）/g, '')
    .replace(/\(.*?\)/g, '')
    .replace(/\s+/g, ' ')
    .trim()
}
