import { useProgressListener } from '../../lib/event'

export function useImportProgress() {
  const progressListener = useProgressListener()

  return {
    processFileNums: progressListener.totalFiles,
    processedFileNums: progressListener.processedFiles,
    currentMessage: progressListener.currentMessage,
    progressPercentage: progressListener.progressPercentage,
    isListening: progressListener.isListening,
    startListening: progressListener.startListen,
    stopListening: progressListener.stopListen,
    resetProgress: progressListener.resetProgress,
  }
}
