import type { DmmPackKeysVm } from '@/lib/command'
import { createQuery } from '@tanstack/svelte-query'
import { commandGetParentDmmPackKeys } from '@/lib/command'
import { queryKeys } from '@/lib/data/queryKeys'

export function useParentDmmPackKeysQuery(workId: number) {
  return createQuery<DmmPackKeysVm | null>({
    queryKey: queryKeys.workParentDmmPack.byId(workId),
    queryFn: () => commandGetParentDmmPackKeys(workId),
  })
}
