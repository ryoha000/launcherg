import type { Env } from '@server/env'
import { os } from '@orpc/server'
import {
  deviceIdSchema,
  deviceRegisterInputSchema,
  deviceRegisterOutputSchema,
  deviceSessionInputSchema,
  deviceWorksListOutputSchema,
  deviceWorksSyncCommitInputSchema,
  deviceWorksSyncCommitOutputSchema,
  deviceWorksSyncPrepareInputSchema,
  deviceWorksSyncPrepareOutputSchema,
} from '@server/shared/schema'

import { z } from 'zod'

const base = os.$context<{
  env: Env
  request: Request
}>()

export const registerDeviceContract = base
  .route({
    method: 'POST',
    path: '/device/register',
  })
  .input(deviceRegisterInputSchema)
  .output(deviceRegisterOutputSchema)

export const createSessionContract = base
  .route({
    method: 'POST',
    path: '/device/session',
  })
  .input(deviceSessionInputSchema)

export const listWorksContract = base
  .route({
    method: 'GET',
    path: '/device/{deviceId}/works',
  })
  .input(z.object({
    deviceId: deviceIdSchema,
  }))
  .output(deviceWorksListOutputSchema)

export const prepareSyncWorksContract = base
  .route({
    method: 'POST',
    path: '/device/{deviceId}/works/sync/prepare',
  })
  .input(deviceWorksSyncPrepareInputSchema.extend({
    deviceId: deviceIdSchema,
  }))
  .output(deviceWorksSyncPrepareOutputSchema)

export const commitSyncWorksContract = base
  .route({
    method: 'POST',
    path: '/device/{deviceId}/works/sync/commit',
  })
  .input(deviceWorksSyncCommitInputSchema.extend({
    deviceId: deviceIdSchema,
  }))
  .output(deviceWorksSyncCommitOutputSchema)

export const appRouter = {
  device: {
    register: registerDeviceContract,
    session: createSessionContract,
    works: {
      list: listWorksContract,
      sync: {
        prepare: prepareSyncWorksContract,
        commit: commitSyncWorksContract,
      },
    },
  },
}
