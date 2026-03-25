import type { RemoteShareSettingsVm } from '@/lib/command'
import { createMutation, createQuery } from '@tanstack/svelte-query'
import {
  commandGetRemoteShareSettings,
  commandGetRemoteShareUrl,
  commandRegisterRemoteShareDevice,
  commandSetRemoteShareSettings,
  commandSyncRemoteShareWorks,
} from '@/lib/command'
import { queryClient } from '@/lib/data/queryClient'
import { queryKeys } from '@/lib/data/queryKeys'

export function useRemoteShareSettingsQuery() {
  return createQuery<RemoteShareSettingsVm>({
    queryKey: queryKeys.remoteShare.settings(),
    queryFn: () => commandGetRemoteShareSettings(),
  })
}

export function useRemoteShareSettingsMutation() {
  return createMutation<RemoteShareSettingsVm, Error, RemoteShareSettingsVm>({
    mutationFn: input => commandSetRemoteShareSettings(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.remoteShare.settings() })
    },
  })
}

export function useRegisterRemoteShareDeviceMutation() {
  return createMutation<RemoteShareSettingsVm, Error, RemoteShareSettingsVm>({
    mutationFn: input => commandRegisterRemoteShareDevice(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.remoteShare.settings() })
    },
  })
}

export function useSyncRemoteShareWorksMutation() {
  return createMutation<RemoteShareSettingsVm, Error, void>({
    mutationFn: () => commandSyncRemoteShareWorks(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.remoteShare.settings() })
    },
  })
}

export function useRemoteShareUrlMutation() {
  return createMutation<string, Error, void>({
    mutationFn: () => commandGetRemoteShareUrl(),
  })
}
