// 汎用ユーティリティ関数

// リクエストIDを生成する純粋関数
export function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// URLからstore_idを抽出する純粋関数
export function extractStoreIdFromUrl(thumbnailUrl: string): string | null {
  // ファイル名の部分のみを対象とする
  const fileName = thumbnailUrl.split('/').pop() || ''
  const rjMatch = fileName.match(/(RJ|VJ|BJ)(\d+)/)
  if (!rjMatch) {
    return null
  }
  return rjMatch[1] + rjMatch[2]
}

// デバッグメッセージを出力する関数
export function debug(debugMode: boolean, message: string, ...args: any[]): void {
  if (debugMode) {
    console.log(`[DLsite Extractor] ${message}`, ...args)
  }
}

// ページが完全に読み込まれるまで待機する関数
export function waitForPageLoad(): Promise<void> {
  return new Promise((resolve) => {
    if (document.readyState === 'complete') {
      // DLsiteは動的コンテンツが多いので少し長めに待機
      setTimeout(resolve, 2000)
    }
    else {
      window.addEventListener('load', () => {
        setTimeout(resolve, 2000)
      })
    }
  })
}
