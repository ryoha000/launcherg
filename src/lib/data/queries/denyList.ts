import { createMutation, createQuery } from '@tanstack/svelte-query'
import { commandDenyListAdd, commandDenyListAll, commandDenyListRemove } from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useDenyListQuery() {
  return createQuery({
    queryKey: queryKeys.denyList.all(),
    queryFn: () => commandDenyListAll(),
  })
}

export function useAddDenyListMutation() {
  return createMutation<unknown, Error, { storeType: number, storeId: string, name: string }>(
    {
      mutationFn: input => commandDenyListAdd(input.storeType, input.storeId, input.name),
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: queryKeys.denyList.all() })
      },
    },
  )
}

export function useRemoveDenyListMutation() {
  return createMutation<unknown, Error, { storeType: number, storeId: string }>(
    {
      mutationFn: input => commandDenyListRemove(input.storeType, input.storeId),
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: queryKeys.denyList.all() })
      },
    },
  )
}
