import type { ProgressLivePayload, ProgressPayload } from './types'
import { useEvent } from './useEvent.svelte'

export function useProgressListener() {
  let totalFiles = $state(0)
  let processedFiles = $state(0)
  let currentMessage = $state('')
  let isListening = $state(false)

  const event = useEvent()

  const startListen = async () => {
    if (isListening)
      return

    await event.startListen('progresslive', (payload: ProgressLivePayload) => {
      if (payload.max) {
        totalFiles = payload.max
      }
      else {
        processedFiles = processedFiles + 1
      }
    })

    await event.startListen('progress', (payload: ProgressPayload) => {
      currentMessage = payload.message
    })

    isListening = true
  }

  const stopListen = () => {
    if (!isListening)
      return

    event.stopAll()
    isListening = false
  }

  const resetProgress = () => {
    totalFiles = 0
    processedFiles = 0
    currentMessage = ''
  }

  const getProgressPercentage = () => {
    if (totalFiles === 0)
      return 0
    return Math.round((processedFiles / totalFiles) * 100)
  }

  return {
    // State getters
    totalFiles: () => totalFiles,
    processedFiles: () => processedFiles,
    currentMessage: () => currentMessage,
    isListening: () => isListening,
    progressPercentage: getProgressPercentage,

    // Actions
    startListen,
    stopListen,
    resetProgress,
  }
}
