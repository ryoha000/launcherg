import type { ProgressLivePayload, ProgressPayload } from '../../lib/event/types'
import { useEvent } from '../../lib/event'

export function useImportProgress() {
  let processFileNums = $state(0)
  let processedFileNums = $state(0)
  let currentMessage = $state('')
  let isListening = $state(false)

  const event = useEvent()

  const startListening = async () => {
    if (isListening)
      return

    await event.startListen('progresslive', (payload: ProgressLivePayload) => {
      if (payload.max) {
        processFileNums = payload.max
      }
      else {
        processedFileNums = processedFileNums + 1
      }
    })

    await event.startListen('progress', (payload: ProgressPayload) => {
      currentMessage = payload.message
    })

    isListening = true
  }

  const stopListening = () => {
    if (!isListening)
      return

    event.stopAll()
    isListening = false
  }

  const resetProgress = () => {
    processFileNums = 0
    processedFileNums = 0
    currentMessage = ''
  }

  const progressPercentage = () => {
    if (processFileNums === 0)
      return 0
    return Math.round((processedFileNums / processFileNums) * 100)
  }

  return {
    processFileNums: () => processFileNums,
    processedFileNums: () => processedFileNums,
    currentMessage: () => currentMessage,
    progressPercentage,
    isListening: () => isListening,
    startListening,
    stopListening,
    resetProgress,
  }
}
