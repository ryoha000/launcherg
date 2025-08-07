// 汎用ユーティリティ関数

// リクエストIDを生成する純粋関数
export function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// デバッグメッセージを出力する関数
export function debug(debugMode: boolean, message: string, ...args: any[]): void {
  if (debugMode) {
    console.log(message, ...args)
  }
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
