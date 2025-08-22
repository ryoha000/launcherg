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

// ExtensionRequest を送信して Promise を返す軽量ヘルパ
export function sendExtensionRequest<TReq>(
  request: TReq,
  serialize: (req: TReq) => any,
): Promise<any> {
  return new Promise((resolve, reject) => {
    try {
      chrome.runtime.sendMessage(serialize(request), (responseJson) => {
        const lastError = chrome.runtime?.lastError
        if (lastError) {
          reject(new Error(lastError.message))
          return
        }
        resolve(responseJson)
      })
    }
    catch (e) {
      reject(e)
    }
  })
}

// タイトル正規化（DMM / DLsite 共通）
// - 角括弧【...】や丸括弧（...）/ (...)、角括弧[...]内の付帯情報を除去
// - 連続空白を単一空白へ圧縮し、前後空白を除去
export function normalizeTitle(raw: string): string {
  return (raw || '')
    .replace(/【.*?】/g, '')
    .replace(/\[.*?\]/g, '')
    .replace(/（.*?）/g, '')
    .replace(/\(.*?\)/g, '')
    .replace(/\s+/g, ' ')
    .trim()
}
