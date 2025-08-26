import { createMutation, createQuery } from '@tanstack/svelte-query'
import { commandWorkOmitAdd, commandWorkOmitAll, commandWorkOmitRemove } from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useWorkOmitQuery() {
  return createQuery({
    queryKey: queryKeys.denyList.all(),
    // workId の配列を返す
    queryFn: () => commandWorkOmitAll(),
  })
}

export function useAddWorkOmitMutation() {
  return createMutation<unknown, Error, { workId: number }>(
    {
      mutationFn: input => commandWorkOmitAdd(input.workId),
      onSuccess: async () => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.denyList.all() }),
          queryClient.invalidateQueries({ queryKey: queryKeys.workDetails.all() }),
        ])
      },
    },
  )
}

export function useRemoveWorkOmitMutation() {
  return createMutation<unknown, Error, { workId: number }>(
    {
      mutationFn: input => commandWorkOmitRemove(input.workId),
      onSuccess: async () => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.denyList.all() }),
          queryClient.invalidateQueries({ queryKey: queryKeys.workDetails.all() }),
        ])
      },
    },
  )
}
