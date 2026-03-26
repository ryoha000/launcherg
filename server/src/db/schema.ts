import { sql } from 'drizzle-orm'
import {
  index,
  integer,
  primaryKey,
  sqliteTable,
  text,
  uniqueIndex,
} from 'drizzle-orm/sqlite-core'

export const devices = sqliteTable(
  'devices',
  {
    deviceId: text('device_id').primaryKey(),
    secretHash: text('secret_hash').notNull(),
    createdAt: text('created_at').notNull().default(sql`CURRENT_TIMESTAMP`),
    updatedAt: text('updated_at').notNull().default(sql`CURRENT_TIMESTAMP`),
    lastSyncedAt: text('last_synced_at'),
  },
  (table) => ({
    secretHashUnique: uniqueIndex('devices_secret_hash_unique').on(table.secretHash),
  }),
)

export const deviceWorkSnapshots = sqliteTable(
  'device_work_snapshots',
  {
    deviceId: text('device_id')
      .notNull()
      .references(() => devices.deviceId, { onDelete: 'cascade' }),
    workId: text('work_id').notNull(),
    title: text('title').notNull(),
    imageKey: text('image_key'),
    thumbnailWidth: integer('thumbnail_width'),
    thumbnailHeight: integer('thumbnail_height'),
    originalPath: text('original_path'),
    erogamescapeId: integer('erogamescape_id'),
    officialUrl: text('official_url'),
    erogamescapeUrl: text('erogamescape_url'),
    seiyaUrl: text('seiya_url'),
    createdAt: text('created_at').notNull().default(sql`CURRENT_TIMESTAMP`),
    updatedAt: text('updated_at').notNull().default(sql`CURRENT_TIMESTAMP`),
  },
  (table) => ({
    primaryKey: primaryKey({ columns: [table.deviceId, table.workId] }),
    deviceIdIndex: index('idx_device_work_snapshots_device_id').on(table.deviceId),
    deviceIdErogamescapeIdIndex: index(
      'idx_device_work_snapshots_device_id_erogamescape_id',
    ).on(table.deviceId, table.erogamescapeId),
  }),
)

export const remoteShareImages = sqliteTable(
  'remote_share_images',
  {
    dedupeKey: text('dedupe_key').primaryKey(),
    imageKey: text('image_key').notNull(),
    createdAt: text('created_at').notNull().default(sql`CURRENT_TIMESTAMP`),
    updatedAt: text('updated_at').notNull().default(sql`CURRENT_TIMESTAMP`),
  },
)
