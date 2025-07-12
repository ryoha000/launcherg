import { listen } from '@tauri-apps/api/event'
import { showErrorToast } from '@/lib/toast'

interface FileDropPayload {
  paths: string[]
  position: { x: number, y: number }
}

export function useFileDrop() {
  const filePaths: string[] = []
  let targetFile = $state<string | undefined>(undefined)

  const targetFileAccessor = () => targetFile

  const appendFilePaths = (paths: string[]) => {
    if (paths.length === 0) {
      return
    }
    filePaths.push(...paths)
  }

  const popToTargetFile = () => {
    if (filePaths.length === 0) {
      targetFile = undefined
      return
    }
    targetFile = filePaths.pop()
  }

  let stopListening: () => void = () => {}
  const startListening = async () => {
    stopListening = await listen<FileDropPayload>('tauri://drag-drop', (event) => {
      const files = event.payload.paths
      const ignoredPaths: string[] = []
      for (const file of files) {
        const exts = ['exe', 'lnk', 'url']
        const isIgnore = exts.every(ext => !file.toLowerCase().endsWith(ext))
        if (isIgnore) {
          ignoredPaths.push(file)
          continue
        }
        appendFilePaths([file])
      }
      if (ignoredPaths.length > 0) {
        showErrorToast(
          `EXEファイルかショートカットファイルをドラッグアンドドロップしてください。フォルダから追加したい場合はサイドバーの Add ボタンから「自動でフォルダから追加」を選択してください。(path: ${ignoredPaths.join(', ')})`,
          10000,
        )
      }
      if (targetFile === undefined) {
        popToTargetFile()
      }
    })
  }

  return {
    targetFileAccessor,
    popToTargetFile,
    startListening,
    stopListening,
  }
}
