import type { DmmPackMarkVm } from '@/lib/command'
import { createMutation, createQuery } from '@tanstack/svelte-query'
import { commandDmmPackAdd, commandDmmPackAll, commandDmmPackRemove } from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useDmmPackQuery() {
  return createQuery<DmmPackMarkVm[], Error>({
    queryKey: queryKeys.dmmPack.all(),
    queryFn: () => commandDmmPackAll(),
  })
}

export function useAddDmmPackMutation() {
  return createMutation<unknown, Error, { storeId: string, name: string }>({
    mutationFn: input => commandDmmPackAdd(input.storeId, input.name),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.dmmPack.all() })
    },
  })
}

export function useRemoveDmmPackMutation() {
  return createMutation<unknown, Error, { storeId: string }>({
    mutationFn: input => commandDmmPackRemove(input.storeId),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.dmmPack.all() })
    },
  })
}
