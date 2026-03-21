import { createMutation, createQuery } from '@tanstack/svelte-query'
import { commandWorkPackAdd, commandWorkPackAll, commandWorkPackRemove } from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useDmmPackQuery() {
  return createQuery<string[], Error>({
    queryKey: queryKeys.dmmPack.all(),
    queryFn: () => commandWorkPackAll(),
  })
}

export function useAddDmmPackMutation() {
  return createMutation<unknown, Error, { workId: string }>(
    {
      mutationFn: input => commandWorkPackAdd(input.workId),
      onSuccess: async () => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.dmmPack.all() }),
          queryClient.invalidateQueries({ queryKey: queryKeys.workDetails.all() }),
        ])
      },
    },
  )
}

export function useRemoveDmmPackMutation() {
  return createMutation<unknown, Error, { workId: string }>(
    {
      mutationFn: input => commandWorkPackRemove(input.workId),
      onSuccess: async () => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.dmmPack.all() }),
          queryClient.invalidateQueries({ queryKey: queryKeys.workDetails.all() }),
        ])
      },
    },
  )
}
