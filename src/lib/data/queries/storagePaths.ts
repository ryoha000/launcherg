import type { StoragePathSettingsVm } from '@/lib/command'
import { createMutation, createQuery } from '@tanstack/svelte-query'
import { commandGetStorageSettings, commandSetStorageSettings } from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useStorageSettingsQuery() {
  return createQuery<StoragePathSettingsVm>({
    queryKey: queryKeys.storagePaths.all(),
    queryFn: () => commandGetStorageSettings(),
  })
}

export function useStorageSettingsMutation() {
  return createMutation<StoragePathSettingsVm, Error, StoragePathSettingsVm>({
    mutationFn: input => commandSetStorageSettings(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.storagePaths.all() })
    },
  })
}
