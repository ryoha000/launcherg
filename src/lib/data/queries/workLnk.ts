import { createQuery } from '@tanstack/svelte-query'
import { commandListWorkLnks } from '@/lib/command'
import { queryKeys } from '@/lib/data/queryKeys'

export function useWorkLnkQuery(workId: number) {
  return createQuery<[number, string][]>({
    queryKey: queryKeys.workLnk.byId(workId),
    queryFn: () => commandListWorkLnks(workId),
  })
}
