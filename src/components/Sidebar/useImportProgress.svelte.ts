import type {
  ImageQueueItemErrorPayload,
  ImageQueueItemPayload,
  ImageQueueWorkerStatusPayload,
  ScanCandidateDiscoveredPayload,
  ScanExploreFinishedPayload,
} from '../../lib/event/types'
import { useEvent } from '../../lib/event'

export type ScanTaskStatus = 'idle' | 'running' | 'done'

export interface ScanProgressState {
  startedAt: number | null
  elapsedSeconds: number
  explore: {
    status: ScanTaskStatus
    discoveredCandidates: number
    currentPath: string | null
    totalCandidates: number | null
  }
  judge: {
    status: ScanTaskStatus
    judgedCount: number
    recognizedCount: number
    totalCandidates: number | null
  }
  images: {
    status: ScanTaskStatus
    processedCount: number
    totalCount: number | null
  }
}

function createInitialState(): ScanProgressState {
  return {
    startedAt: null,
    elapsedSeconds: 0,
    explore: {
      status: 'idle',
      discoveredCandidates: 0,
      currentPath: null,
      totalCandidates: null,
    },
    judge: {
      status: 'idle',
      judgedCount: 0,
      recognizedCount: 0,
      totalCandidates: null,
    },
    images: {
      status: 'idle',
      processedCount: 0,
      totalCount: null,
    },
  }
}

export function useImportProgress() {
  let progress = $state<ScanProgressState>(createInitialState())
  let isListening = $state(false)
  let timerId: ReturnType<typeof setInterval> | null = null

  const event = useEvent()

  const syncElapsed = () => {
    if (progress.startedAt === null) {
      progress.elapsedSeconds = 0
      return
    }

    progress.elapsedSeconds = Math.max(
      0,
      Math.floor((Date.now() - progress.startedAt) / 1000),
    )
  }

  const stopElapsedTimer = () => {
    if (timerId !== null) {
      clearInterval(timerId)
      timerId = null
    }
  }

  const startElapsedTimer = () => {
    stopElapsedTimer()
    syncElapsed()
    timerId = setInterval(syncElapsed, 1000)
  }

  const resetProgress = () => {
    stopElapsedTimer()
    progress = createInitialState()
  }

  const beginScan = () => {
    resetProgress()
    progress.startedAt = Date.now()
    progress.explore.status = 'running'
    progress.judge.status = 'running'
    progress.images.status = 'idle'
    startElapsedTimer()
  }

  const finishScan = () => {
    syncElapsed()
    stopElapsedTimer()
  }

  const completeJudgeIfReady = () => {
    if (
      progress.explore.status === 'done'
      && progress.judge.totalCandidates !== null
      && progress.judge.judgedCount >= progress.judge.totalCandidates
    ) {
      progress.judge.status = 'done'
    }
  }

  const completeImagesIfReady = () => {
    if (
      progress.images.status === 'running'
      && progress.images.totalCount !== null
      && progress.images.processedCount >= progress.images.totalCount
    ) {
      // ワーカー完了イベントを待つが、件数が揃っているなら完了扱いでもよい
      progress.images.status = 'done'
    }
  }

  const startListening = async () => {
    if (isListening) {
      return
    }

    await event.startListen('scanCandidateDiscovered', (payload: ScanCandidateDiscoveredPayload) => {
      progress.explore.status = 'running'
      progress.explore.discoveredCandidates = payload.count
      progress.explore.currentPath = payload.path
    })

    await event.startListen('scanExploreFinished', (payload: ScanExploreFinishedPayload) => {
      progress.explore.status = 'done'
      progress.explore.totalCandidates = payload.totalCandidates
      progress.judge.totalCandidates = payload.totalCandidates
      completeJudgeIfReady()
    })

    await event.startListen('scanEnrichResult', (payload) => {
      progress.judge.judgedCount += 1
      if (payload.status === 'resolved') {
        progress.judge.recognizedCount += 1
      }
      completeJudgeIfReady()
    })

    await event.startListen('imageQueueWorkerStarted', (payload: ImageQueueWorkerStatusPayload) => {
      progress.images.status = 'running'
      if (payload.totalCount !== undefined && payload.totalCount !== null) {
        progress.images.totalCount = payload.totalCount
      }
    })

    await event.startListen('imageQueueItemSucceeded', (_payload: ImageQueueItemPayload) => {
      progress.images.processedCount += 1
      completeImagesIfReady()
    })

    await event.startListen('imageQueueItemFailed', (_payload: ImageQueueItemErrorPayload) => {
      progress.images.processedCount += 1
      completeImagesIfReady()
    })

    await event.startListen('imageQueueWorkerFinished', (payload: ImageQueueWorkerStatusPayload) => {
      progress.images.status = 'done'
      if (payload.totalCount !== undefined && payload.totalCount !== null) {
        progress.images.totalCount = payload.totalCount
      }
    })

    isListening = true
  }

  const stopListening = () => {
    if (!isListening) {
      return
    }

    stopElapsedTimer()
    event.stopAll()
    isListening = false
  }

  return {
    progress: () => progress,
    isListening: () => isListening,
    startListening,
    stopListening,
    resetProgress,
    beginScan,
    finishScan,
  }
}
