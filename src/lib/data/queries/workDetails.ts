import type { WorkDetailsVm } from '@/lib/command'
import { createQuery } from '@tanstack/svelte-query'
import { commandGetWorkDetailsAll, commandGetWorkDetailsByWorkId } from '@/lib/command'
import { queryKeys } from '@/lib/data/queryKeys'

export function useWorkDetailsAllQuery() {
  return createQuery<WorkDetailsVm[]>({
    queryKey: queryKeys.workDetails.all(),
    queryFn: () => commandGetWorkDetailsAll(),
  })
}

export function useWorkDetailsByWorkIdQuery(workId: number) {
  return createQuery<WorkDetailsVm | null>({
    queryKey: queryKeys.workDetails.byId(workId),
    queryFn: () => commandGetWorkDetailsByWorkId(workId),
  })
}
