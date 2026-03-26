import { eq, inArray, sql } from 'drizzle-orm'

import { createDbClient } from '@server/db/client'
import {
  deviceWorkSnapshots,
  devices,
  remoteShareImages,
} from '@server/db/schema'

import type {
  DeviceWorksListItem,
  RemoteShareWorkInput,
} from '@server/shared/schema'

export interface DeviceRecord {
  deviceId: string
  secretHash: string
  lastSyncedAt: string | null
}

export interface DeviceSnapshotRow {
  workId: string
  title: string
  imageKey: string | null
  thumbnailWidth: number | null
  thumbnailHeight: number | null
  erogamescapeId: number | null
  officialUrl: string | null
  erogamescapeUrl: string | null
  seiyaUrl: string | null
}

export interface RemoteShareImageRow {
  dedupeKey: string
  imageKey: string
}

export async function findDeviceBySecretHash(
  db: D1Database,
  secretHash: string,
): Promise<DeviceRecord | null> {
  const database = createDbClient(db)
  const row = await database
    .select({
      deviceId: devices.deviceId,
      secretHash: devices.secretHash,
      lastSyncedAt: devices.lastSyncedAt,
    })
    .from(devices)
    .where(eq(devices.secretHash, secretHash))
    .get()

  if (!row) {
    return null
  }

  return {
    deviceId: row.deviceId,
    secretHash: row.secretHash,
    lastSyncedAt: row.lastSyncedAt,
  }
}

export async function findDeviceById(
  db: D1Database,
  deviceId: string,
): Promise<DeviceRecord | null> {
  const database = createDbClient(db)
  const row = await database
    .select({
      deviceId: devices.deviceId,
      secretHash: devices.secretHash,
      lastSyncedAt: devices.lastSyncedAt,
    })
    .from(devices)
    .where(eq(devices.deviceId, deviceId))
    .get()

  if (!row) {
    return null
  }

  return {
    deviceId: row.deviceId,
    secretHash: row.secretHash,
    lastSyncedAt: row.lastSyncedAt,
  }
}

export async function insertDevice(
  db: D1Database,
  deviceId: string,
  secretHash: string,
): Promise<void> {
  const database = createDbClient(db)
  await database.insert(devices).values({
    deviceId,
    secretHash,
  }).run()
}

export async function clearDeviceSnapshots(
  db: D1Database,
  deviceId: string,
): Promise<void> {
  const database = createDbClient(db)
  await database.delete(deviceWorkSnapshots).where(eq(deviceWorkSnapshots.deviceId, deviceId)).run()
}

export async function upsertDeviceSnapshots(
  db: D1Database,
  deviceId: string,
  works: RemoteShareWorkInput[],
  imageKeys: Map<string, string>,
  syncedAt: string,
): Promise<void> {
  const database = createDbClient(db)
  const statements = works.map((work) => {
    const dedupeKey = remoteShareDedupeKeyForWork(work)
    const imageKey = imageKeys.get(dedupeKey) ?? null

    return database
      .insert(deviceWorkSnapshots)
      .values({
        deviceId,
        workId: work.workId,
        erogamescapeId: work.erogamescapeId ?? null,
        title: work.title,
        officialUrl: work.officialUrl ?? null,
        erogamescapeUrl: work.erogamescapeUrl ?? null,
        seiyaUrl: work.seiyaUrl ?? null,
        imageKey,
        thumbnailWidth: work.thumbnail?.width ?? null,
        thumbnailHeight: work.thumbnail?.height ?? null,
        updatedAt: syncedAt,
      })
      .onConflictDoUpdate({
        target: [deviceWorkSnapshots.deviceId, deviceWorkSnapshots.workId],
        set: {
          erogamescapeId: work.erogamescapeId ?? null,
          title: work.title,
          officialUrl: work.officialUrl ?? null,
          erogamescapeUrl: work.erogamescapeUrl ?? null,
          seiyaUrl: work.seiyaUrl ?? null,
          imageKey,
          thumbnailWidth: work.thumbnail?.width ?? null,
          thumbnailHeight: work.thumbnail?.height ?? null,
          updatedAt: syncedAt,
        },
      })
  })

  if (statements.length > 0) {
    await database.batch(statements as [typeof statements[number], ...typeof statements[number][]])
  }

  await database
    .update(devices)
    .set({
      lastSyncedAt: syncedAt,
      updatedAt: syncedAt,
    })
    .where(eq(devices.deviceId, deviceId))
    .run()
}

export async function listDeviceSnapshots(
  db: D1Database,
  deviceId: string,
): Promise<DeviceSnapshotRow[]> {
  const database = createDbClient(db)
  const rows = await database
    .select({
      workId: deviceWorkSnapshots.workId,
      title: deviceWorkSnapshots.title,
      imageKey: deviceWorkSnapshots.imageKey,
      thumbnailWidth: deviceWorkSnapshots.thumbnailWidth,
      thumbnailHeight: deviceWorkSnapshots.thumbnailHeight,
      erogamescapeId: deviceWorkSnapshots.erogamescapeId,
      officialUrl: deviceWorkSnapshots.officialUrl,
      erogamescapeUrl: deviceWorkSnapshots.erogamescapeUrl,
      seiyaUrl: deviceWorkSnapshots.seiyaUrl,
    })
    .from(deviceWorkSnapshots)
    .where(eq(deviceWorkSnapshots.deviceId, deviceId))
    .orderBy(sql`${deviceWorkSnapshots.title} COLLATE NOCASE`)
    .all()

  return rows.map(row => ({
    workId: row.workId,
    title: row.title,
    imageKey: row.imageKey,
    thumbnailWidth: row.thumbnailWidth,
    thumbnailHeight: row.thumbnailHeight,
    erogamescapeId: row.erogamescapeId,
    officialUrl: row.officialUrl,
    erogamescapeUrl: row.erogamescapeUrl,
    seiyaUrl: row.seiyaUrl,
  }))
}

export function toPublicWorkItem(
  deviceId: string,
  row: DeviceSnapshotRow,
): DeviceWorksListItem {
  return {
    workId: row.workId,
    title: row.title,
    imageUrl: row.imageKey
      ? `/api/device/${encodeURIComponent(deviceId)}/images/${encodeURIComponent(row.imageKey)}`
      : undefined,
    width: row.thumbnailWidth ?? undefined,
    height: row.thumbnailHeight ?? undefined,
    officialUrl: row.officialUrl ?? undefined,
    erogamescapeUrl: row.erogamescapeUrl ?? undefined,
    seiyaUrl: row.seiyaUrl ?? undefined,
  }
}

export async function findRemoteShareImageByDedupeKey(
  db: D1Database,
  dedupeKey: string,
): Promise<RemoteShareImageRow | null> {
  const database = createDbClient(db)
  const row = await database
    .select({
      dedupeKey: remoteShareImages.dedupeKey,
      imageKey: remoteShareImages.imageKey,
    })
    .from(remoteShareImages)
    .where(eq(remoteShareImages.dedupeKey, dedupeKey))
    .get()

  if (!row) {
    return null
  }

  return {
    dedupeKey: row.dedupeKey,
    imageKey: row.imageKey,
  }
}

export async function listRemoteShareImagesByDedupeKeys(
  db: D1Database,
  dedupeKeys: string[],
): Promise<Map<string, string>> {
  const database = createDbClient(db)
  const results = new Map<string, string>()
  const uniqueKeys = Array.from(new Set(dedupeKeys))

  for (let index = 0; index < uniqueKeys.length; index += 100) {
    const chunk = uniqueKeys.slice(index, index + 100)
    if (chunk.length === 0) {
      continue
    }

    const rows = await database
      .select({
        dedupeKey: remoteShareImages.dedupeKey,
        imageKey: remoteShareImages.imageKey,
      })
      .from(remoteShareImages)
      .where(inArray(remoteShareImages.dedupeKey, chunk))
      .all()

    for (const row of rows) {
      results.set(row.dedupeKey, row.imageKey)
    }
  }

  return results
}

export async function insertRemoteShareImage(
  db: D1Database,
  dedupeKey: string,
  imageKey: string,
): Promise<void> {
  const database = createDbClient(db)
  await database
    .insert(remoteShareImages)
    .values({
      dedupeKey,
      imageKey,
    })
    .onConflictDoNothing()
    .run()
}

export function remoteShareDedupeKeyForWork(work: RemoteShareWorkInput): string {
  if (work.erogamescapeId !== null && work.erogamescapeId !== undefined) {
    return `egs:${work.erogamescapeId}`
  }

  return `work:${work.workId}`
}

export function remoteShareImageKeyForDedupeKey(dedupeKey: string): string {
  return `remote-share/${encodeURIComponent(dedupeKey)}/thumbnail`
}
