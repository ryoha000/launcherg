import { readFile } from '@tauri-apps/plugin-fs'

export function useChunk() {
  const currentChunkId = 0
  // chunk id は頭につける都合上8bitで表現できるようにする
  const chunkIdMask = 0xFF
  const CHUNK_SIZE = 16 * 1024 // 16KB
  const CHUNK_HEADER_SIZE = 2 // [chunkId: 1byte][index: 1byte]
  const CHUNK_DATA_SIZE = CHUNK_SIZE - CHUNK_HEADER_SIZE
  const createNewChunkId = () => {
    return (currentChunkId + 1) & chunkIdMask
  }

  const createChunks = async (filePath: string) => {
    // ファイルをバイナリとして読み込む
    const data = await readFile(filePath)
    const lowerCasePath = filePath.toLowerCase()
    // MIME タイプを推定 (ここでは ".png" の場合 "image/png" としていますが、他の形式もサポートする場合は調整が必要)
    const mimeType = (function () {
      if (lowerCasePath.endsWith('.png'))
        return 'image/png'
      if (lowerCasePath.endsWith('.jpg') || lowerCasePath.endsWith('.jpeg'))
        return 'image/jpeg'
      if (lowerCasePath.endsWith('.gif'))
        return 'image/gif'
      if (lowerCasePath.endsWith('.webp'))
        return 'image/webp'
      throw new Error('Unsupported file type')
    })()
    const chunkId = createNewChunkId()

    const totalChunkLength = Math.ceil(data.byteLength / CHUNK_DATA_SIZE)
    const chunkArray: Uint8Array[] = []
    for (let i = 0; i < totalChunkLength; i++) {
      chunkArray[i] = new Uint8Array([
        chunkId,
        i,
        ...data.slice(
          i * CHUNK_DATA_SIZE,
          Math.min((i + 1) * CHUNK_DATA_SIZE, data.byteLength),
        ),
      ])
    }

    return [{ chunkId, mimeType, totalChunkLength }, chunkArray] as const
  }

  return { createChunks }
}
