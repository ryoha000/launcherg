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
}

export interface RemoteShareImageRow {
  dedupeKey: string
  imageKey: string
}

export async function findDeviceBySecretHash(
  db: D1Database,
  secretHash: string,
): Promise<DeviceRecord | null> {
  const row = await db
    .prepare(
      `SELECT device_id, secret_hash, last_synced_at
       FROM devices
       WHERE secret_hash = ?1
       LIMIT 1`,
    )
    .bind(secretHash)
    .first<{
      device_id: string
      secret_hash: string
      last_synced_at: string | null
    }>()

  if (!row) {
    return null
  }

  return {
    deviceId: row.device_id,
    secretHash: row.secret_hash,
    lastSyncedAt: row.last_synced_at,
  }
}

export async function findDeviceById(
  db: D1Database,
  deviceId: string,
): Promise<DeviceRecord | null> {
  const row = await db
    .prepare(
      `SELECT device_id, secret_hash, last_synced_at
       FROM devices
       WHERE device_id = ?1
       LIMIT 1`,
    )
    .bind(deviceId)
    .first<{
      device_id: string
      secret_hash: string
      last_synced_at: string | null
    }>()

  if (!row) {
    return null
  }

  return {
    deviceId: row.device_id,
    secretHash: row.secret_hash,
    lastSyncedAt: row.last_synced_at,
  }
}

export async function insertDevice(
  db: D1Database,
  deviceId: string,
  secretHash: string,
): Promise<void> {
  await db
    .prepare(
      `INSERT INTO devices (device_id, secret_hash)
       VALUES (?1, ?2)`,
    )
    .bind(deviceId, secretHash)
    .run()
}

export async function clearDeviceSnapshots(
  db: D1Database,
  deviceId: string,
): Promise<void> {
  await db
    .prepare('DELETE FROM device_work_snapshots WHERE device_id = ?1')
    .bind(deviceId)
    .run()
}

export async function upsertDeviceSnapshots(
  db: D1Database,
  deviceId: string,
  works: RemoteShareWorkInput[],
  imageKeys: Map<string, string>,
  syncedAt: string,
): Promise<void> {
  const statements = works.map((work) => {
    const dedupeKey = remoteShareDedupeKeyForWork(work)
    const imageKey = imageKeys.get(dedupeKey) ?? null
    return db
      .prepare(
        `INSERT INTO device_work_snapshots (
          device_id,
          work_id,
          erogamescape_id,
          title,
          image_key,
          thumbnail_width,
          thumbnail_height,
          updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(device_id, work_id) DO UPDATE SET
          erogamescape_id = excluded.erogamescape_id,
          title = excluded.title,
          image_key = excluded.image_key,
          thumbnail_width = excluded.thumbnail_width,
          thumbnail_height = excluded.thumbnail_height,
          updated_at = excluded.updated_at`,
      )
      .bind(
        deviceId,
        work.workId,
        work.erogamescapeId ?? null,
        work.title,
        imageKey,
        work.thumbnail?.width ?? null,
        work.thumbnail?.height ?? null,
        syncedAt,
      )
  })

  if (statements.length > 0) {
    await db.batch(statements)
  }

  await db
    .prepare(
      `UPDATE devices
       SET last_synced_at = ?2,
           updated_at = ?2
       WHERE device_id = ?1`,
    )
    .bind(deviceId, syncedAt)
    .run()
}

export async function listDeviceSnapshots(
  db: D1Database,
  deviceId: string,
): Promise<DeviceSnapshotRow[]> {
  const result = await db
    .prepare(
      `SELECT
         work_id,
         title,
         image_key,
         thumbnail_width,
         thumbnail_height,
         erogamescape_id
       FROM device_work_snapshots
       WHERE device_id = ?1
       ORDER BY title COLLATE NOCASE ASC`,
    )
    .bind(deviceId)
    .all<{
      work_id: string
      title: string
      image_key: string | null
      thumbnail_width: number | null
      thumbnail_height: number | null
      erogamescape_id: number | null
    }>()

  return result.results.map(row => ({
    workId: row.work_id,
    title: row.title,
    imageKey: row.image_key,
    thumbnailWidth: row.thumbnail_width,
    thumbnailHeight: row.thumbnail_height,
    erogamescapeId: row.erogamescape_id,
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
  }
}

export async function findRemoteShareImageByDedupeKey(
  db: D1Database,
  dedupeKey: string,
): Promise<RemoteShareImageRow | null> {
  const row = await db
    .prepare(
      `SELECT dedupe_key, image_key
       FROM remote_share_images
       WHERE dedupe_key = ?1
       LIMIT 1`,
    )
    .bind(dedupeKey)
    .first<{
      dedupe_key: string
      image_key: string
    }>()

  if (!row) {
    return null
  }

  return {
    dedupeKey: row.dedupe_key,
    imageKey: row.image_key,
  }
}

export async function listRemoteShareImagesByDedupeKeys(
  db: D1Database,
  dedupeKeys: string[],
): Promise<Map<string, string>> {
  const results = new Map<string, string>()
  const uniqueKeys = Array.from(new Set(dedupeKeys))

  for (let index = 0; index < uniqueKeys.length; index += 100) {
    const chunk = uniqueKeys.slice(index, index + 100)
    if (chunk.length === 0) {
      continue
    }

    const placeholders = chunk.map((_, itemIndex) => `?${itemIndex + 1}`).join(', ')
    const rows = await db
      .prepare(
        `SELECT dedupe_key, image_key
         FROM remote_share_images
         WHERE dedupe_key IN (${placeholders})`,
      )
      .bind(...chunk)
      .all<{
        dedupe_key: string
        image_key: string
      }>()

    for (const row of rows.results) {
      results.set(row.dedupe_key, row.image_key)
    }
  }

  return results
}

export async function insertRemoteShareImage(
  db: D1Database,
  dedupeKey: string,
  imageKey: string,
): Promise<void> {
  await db
    .prepare(
      `INSERT OR IGNORE INTO remote_share_images (dedupe_key, image_key)
       VALUES (?1, ?2)`,
    )
    .bind(dedupeKey, imageKey)
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
