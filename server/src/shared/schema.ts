import { z } from 'zod'

export const deviceIdSchema = z.string().uuid()

export const deviceSecretSchema = z
  .string()
  .trim()
  .min(4, 'deviceSecret must be at least 4 characters')
  .max(256, 'deviceSecret is too long')

export const thumbnailInputSchema = z.object({
  contentType: z.string().trim().min(1),
  base64: z.string().trim().min(1),
  width: z.number().int().positive().optional(),
  height: z.number().int().positive().optional(),
})

export const remoteShareWorkInputSchema = z.object({
  workId: z.string().trim().min(1),
  title: z.string().trim().min(1),
  erogamescapeId: z.number().int().positive().nullable().optional(),
  officialUrl: z.string().trim().url().nullable().optional(),
  erogamescapeUrl: z.string().trim().url().nullable().optional(),
  seiyaUrl: z.string().trim().url().nullable().optional(),
  thumbnail: z.object({
    contentType: z.string().trim().min(1),
    width: z.number().int().positive().nullable().optional(),
    height: z.number().int().positive().nullable().optional(),
  }).nullable().optional(),
})

export const deviceRegisterInputSchema = z.object({
  deviceSecret: deviceSecretSchema,
})

export const deviceRegisterOutputSchema = z.object({
  deviceId: deviceIdSchema,
})

export const deviceSessionInputSchema = z.object({
  deviceId: deviceIdSchema,
  deviceSecret: deviceSecretSchema,
})

export const deviceWorksSyncPrepareInputSchema = z.object({
  deviceSecret: deviceSecretSchema,
  works: z.array(remoteShareWorkInputSchema),
})

export const deviceWorksListItemSchema = z.object({
  workId: z.string(),
  title: z.string(),
  imageUrl: z.string().optional(),
  width: z.number().int().positive().optional(),
  height: z.number().int().positive().optional(),
  officialUrl: z.string().url().nullable().optional(),
  erogamescapeUrl: z.string().url().nullable().optional(),
  seiyaUrl: z.string().url().nullable().optional(),
})

export const remoteShareUploadTargetSchema = z.object({
  workId: z.string(),
  dedupeKey: z.string(),
  imageKey: z.string(),
  uploadUrl: z.string().url(),
  contentType: z.string(),
})

export const remoteShareUploadedImageSchema = z.object({
  dedupeKey: z.string(),
  imageKey: z.string(),
})

export const deviceWorksSyncCommitInputSchema = z.object({
  deviceSecret: deviceSecretSchema,
  works: z.array(remoteShareWorkInputSchema),
  uploadedImages: z.array(remoteShareUploadedImageSchema),
})

export const deviceWorksSyncPrepareOutputSchema = z.object({
  deviceId: deviceIdSchema,
  uploadTargets: z.array(remoteShareUploadTargetSchema),
})

export const deviceWorksListOutputSchema = z.object({
  deviceId: deviceIdSchema,
  lastSyncedAt: z.string().nullable(),
  works: z.array(deviceWorksListItemSchema),
})

export const deviceWorksSyncCommitOutputSchema = z.object({
  deviceId: deviceIdSchema,
  syncedCount: z.number().int().nonnegative(),
  lastSyncedAt: z.string(),
})

export type DeviceRegisterInput = z.infer<typeof deviceRegisterInputSchema>
export type DeviceRegisterOutput = z.infer<typeof deviceRegisterOutputSchema>
export type DeviceSessionInput = z.infer<typeof deviceSessionInputSchema>
export type DeviceWorksSyncPrepareInput = z.infer<typeof deviceWorksSyncPrepareInputSchema>
export type DeviceWorksSyncCommitInput = z.infer<typeof deviceWorksSyncCommitInputSchema>
export type DeviceWorksSyncPrepareOutput = z.infer<typeof deviceWorksSyncPrepareOutputSchema>
export type DeviceWorksSyncCommitOutput = z.infer<typeof deviceWorksSyncCommitOutputSchema>
export type DeviceWorksListItem = z.infer<typeof deviceWorksListItemSchema>
export type DeviceWorksListOutput = z.infer<typeof deviceWorksListOutputSchema>
export type RemoteShareWorkInput = z.infer<typeof remoteShareWorkInputSchema>
export type RemoteShareUploadTarget = z.infer<typeof remoteShareUploadTargetSchema>
export type RemoteShareUploadedImage = z.infer<typeof remoteShareUploadedImageSchema>
