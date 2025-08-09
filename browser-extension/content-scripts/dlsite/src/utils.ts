// DLsite固有のユーティリティ関数

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
