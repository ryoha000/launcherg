import type { WorkDetailsVm } from '@/lib/command'
import { createQuery } from '@tanstack/svelte-query'
import { commandGetWorkDetailsAll } from '@/lib/command'
import { queryKeys } from '@/lib/data/queryKeys'

export function useWorkDetailsAllQuery() {
  return createQuery<WorkDetailsVm[]>({
    queryKey: queryKeys.workDetails.all(),
    queryFn: () => commandGetWorkDetailsAll(),
  })
}
