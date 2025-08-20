// DLsite固有のユーティリティ関数

// URLからstore_idを抽出する純粋関数
export function extractStoreIdFromUrl(imageUrl: string): string | null {
  // ファイル名の部分のみを対象とする
  const fileName = imageUrl.split('/').pop() || ''
  const rjMatch = fileName.match(/(RJ|VJ|BJ)(\d+)/)
  if (!rjMatch) {
    return null
  }
  return rjMatch[1] + rjMatch[2]
}
