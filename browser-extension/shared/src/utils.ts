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
