import { listen } from '@tauri-apps/api/event'

export function useImportProgress() {
  let processFileNums = $state(0)
  let processedFileNums = $state(0)
  let unlistenProgressLive: (() => void) | null = null

  const startListening = async () => {
    unlistenProgressLive = await listen<{ max: number | null }>(
      'progresslive',
      (event) => {
        if (event.payload.max) {
          processFileNums = event.payload.max
        }
        else {
          processedFileNums = processedFileNums + 1
        }
      },
    )
  }

  const stopListening = () => {
    if (unlistenProgressLive) {
      unlistenProgressLive()
      unlistenProgressLive = null
    }
  }

  const resetProgress = () => {
    processFileNums = 0
    processedFileNums = 0
  }

  return {
    processFileNums: () => processFileNums,
    processedFileNums: () => processedFileNums,
    startListening,
    stopListening,
    resetProgress,
  }
}
