import type { ProgressLivePayload, ProgressPayload } from './types'
import { useMultiEventListener } from './useMultiEventListener.svelte'

export interface ProgressState {
  totalFiles: number
  processedFiles: number
  currentMessage: string
  isListening: boolean
}

export function useProgressListener() {
  let totalFiles = $state(0)
  let processedFiles = $state(0)
  let currentMessage = $state('')
  let isListening = $state(false)

  const eventListener = useMultiEventListener()

  const startListen = async () => {
    if (isListening)
      return

    await eventListener.startListen('progresslive', (payload: ProgressLivePayload) => {
      if (payload.max) {
        totalFiles = payload.max
      }
      else {
        processedFiles = processedFiles + 1
      }
    })

    await eventListener.startListen('progress', (payload: ProgressPayload) => {
      currentMessage = payload.message
    })

    isListening = true
  }

  const stopListen = () => {
    if (!isListening)
      return

    eventListener.stopAllListeners()
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
