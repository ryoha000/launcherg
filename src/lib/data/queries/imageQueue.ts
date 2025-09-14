import type { ImageSaveQueueRowVm } from '@/lib/command'
import { createQuery } from '@tanstack/svelte-query'
import { commandGetImageSaveQueue } from '@/lib/command'
import { queryKeys } from '@/lib/data/queryKeys'

export function useImageQueueQuery(unfinished: boolean) {
  return createQuery<ImageSaveQueueRowVm[]>({
    queryKey: unfinished ? queryKeys.imageQueue.unfinished() : queryKeys.imageQueue.finished(),
    queryFn: () => unfinished
      ? commandGetImageSaveQueue({ limit: 500, status: 'unfinished' })
      : commandGetImageSaveQueue({ limit: 500, status: 'finished' }),
  })
}
