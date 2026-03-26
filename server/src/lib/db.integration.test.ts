import Database from 'better-sqlite3'
import { readFile } from 'node:fs/promises'
import { join } from 'node:path'

import {
  clearDeviceSnapshots,
  findDeviceById,
  findDeviceBySecretHash,
  findRemoteShareImageByDedupeKey,
  insertDevice,
  insertRemoteShareImage,
  listDeviceSnapshots,
  listRemoteShareImagesByDedupeKeys,
  remoteShareDedupeKeyForWork,
  upsertDeviceSnapshots,
} from '@server/lib/db'

import { describe, expect, it } from 'vitest'

type BetterSqliteDatabase = ReturnType<typeof Database>

class BetterSqliteD1PreparedStatement {
  readonly isReader: boolean

  constructor(
    private readonly database: BetterSqliteDatabase,
    private readonly query: string,
    private readonly values: unknown[] = [],
  ) {
    this.isReader = this.database.prepare(this.query).reader
  }

  bind(...values: unknown[]) {
    return new BetterSqliteD1PreparedStatement(this.database, this.query, values)
  }

  async run() {
    const statement = this.database.prepare(this.query)
    const info = statement.run.apply(statement, this.values as never[])
    return {
      success: true as const,
      results: [],
      meta: {
        duration: 0,
        size_after: 0,
        rows_read: 0,
        rows_written: info.changes,
        last_row_id: Number(info.lastInsertRowid ?? 0),
        changed_db: info.changes > 0,
        changes: info.changes,
      },
    }
  }

  async all() {
    const statement = this.database.prepare(this.query)
    const rows = statement.all.apply(statement, this.values as never[])
    return {
      success: true as const,
      results: rows,
      meta: {
        duration: 0,
        size_after: 0,
        rows_read: rows.length,
        rows_written: 0,
        last_row_id: 0,
        changed_db: false,
        changes: 0,
      },
    }
  }

  async raw(options?: { columnNames?: boolean }) {
    const statement = this.database.prepare(this.query).raw()
    const rows = statement.all.apply(statement, this.values as never[]) as unknown[][]

    if (options?.columnNames) {
      return [
        statement.columns().map(column => column.name),
        ...rows,
      ] as [string[], ...unknown[][]]
    }

    return rows
  }

  async first(columnName?: string) {
    const statement = this.database.prepare(this.query)
    const row = statement.get.apply(statement, this.values as never[]) as
      | Record<string, unknown>
      | undefined

    if (!row) {
      return null
    }

    if (!columnName) {
      return row
    }

    return (row as Record<string, unknown>)[columnName] ?? null
  }
}

class BetterSqliteD1Database {
  constructor(private readonly database: BetterSqliteDatabase) {}

  prepare(query: string) {
    return new BetterSqliteD1PreparedStatement(this.database, query)
  }

  async batch(statements: BetterSqliteD1PreparedStatement[]) {
    const results = []
    for (const statement of statements) {
      results.push(statement.isReader ? await statement.all() : await statement.run())
    }
    return results
  }

  async exec(query: string) {
    this.database.exec(query)
    return {
      count: query.split(';').filter(Boolean).length,
      duration: 0,
    }
  }

  withSession() {
    return this as unknown as D1DatabaseSession
  }

  async dump() {
    return new ArrayBuffer(0)
  }
}

async function createMigratedDatabase() {
  const sqlite = new Database(':memory:')
  sqlite.pragma('foreign_keys = ON')

  const database = new BetterSqliteD1Database(sqlite)
  const migrationDir = join(process.cwd(), 'migrations')
  const migrationFiles = [
    '0001_initial.sql',
    '0002_remote_share.sql',
    '0003_remote_share_work_links.sql',
  ]

  for (const fileName of migrationFiles) {
    const sqlText = await readFile(join(migrationDir, fileName), 'utf8')
    await database.exec(sqlText)
  }

  return database as unknown as D1Database
}

describe('db helpers on migrated schema', () => {
  it('migration 済み D1 で CRUD と upsert が一致する', async () => {
    const db = await createMigratedDatabase()

    await insertDevice(db, 'device-1', 'secret-hash-1')
    expect(await findDeviceById(db, 'device-1')).toEqual({
      deviceId: 'device-1',
      secretHash: 'secret-hash-1',
      lastSyncedAt: null,
    })
    expect(await findDeviceBySecretHash(db, 'secret-hash-1')).toEqual({
      deviceId: 'device-1',
      secretHash: 'secret-hash-1',
      lastSyncedAt: null,
    })

    await expect(insertDevice(db, 'device-2', 'secret-hash-1')).rejects.toThrow()

    await insertRemoteShareImage(db, 'egs:123', 'remote-share/egs%3A123/thumbnail')
    await insertRemoteShareImage(db, 'egs:123', 'remote-share/egs%3A123/thumbnail')
    expect(await findRemoteShareImageByDedupeKey(db, 'egs:123')).toEqual({
      dedupeKey: 'egs:123',
      imageKey: 'remote-share/egs%3A123/thumbnail',
    })
    expect(
      await listRemoteShareImagesByDedupeKeys(db, ['egs:123', 'egs:123', 'missing']),
    ).toEqual(new Map([['egs:123', 'remote-share/egs%3A123/thumbnail']]))

    const firstWork = {
      workId: 'work-1',
      title: 'Alpha',
      erogamescapeId: 123,
      officialUrl: 'https://example.com/official',
      erogamescapeUrl: 'https://example.com/egs',
      seiyaUrl: 'https://example.com/seiya',
      thumbnail: {
        contentType: 'image/jpeg',
        width: 320,
        height: 180,
      },
    }

    const firstImageKey = 'remote-share/egs%3A123/thumbnail'
    const firstSyncedAt = '2026-03-26T00:00:00.000Z'

    await upsertDeviceSnapshots(
      db,
      'device-1',
      [firstWork],
      new Map([[remoteShareDedupeKeyForWork(firstWork), firstImageKey]]),
      firstSyncedAt,
    )

    expect(await listDeviceSnapshots(db, 'device-1')).toEqual([
      {
        workId: 'work-1',
        title: 'Alpha',
        imageKey: firstImageKey,
        thumbnailWidth: 320,
        thumbnailHeight: 180,
        erogamescapeId: 123,
        officialUrl: 'https://example.com/official',
        erogamescapeUrl: 'https://example.com/egs',
        seiyaUrl: 'https://example.com/seiya',
      },
    ])

    expect(await findDeviceById(db, 'device-1')).toEqual({
      deviceId: 'device-1',
      secretHash: 'secret-hash-1',
      lastSyncedAt: firstSyncedAt,
    })

    await clearDeviceSnapshots(db, 'device-1')
    expect(await listDeviceSnapshots(db, 'device-1')).toEqual([])

    const updatedWork = {
      ...firstWork,
      title: 'Beta',
      thumbnail: {
        contentType: 'image/jpeg',
        width: 640,
        height: 360,
      },
    }
    const updatedImageKey = 'remote-share/egs%3A123/thumbnail-v2'
    const updatedSyncedAt = '2026-03-26T00:10:00.000Z'

    await upsertDeviceSnapshots(
      db,
      'device-1',
      [updatedWork],
      new Map([[remoteShareDedupeKeyForWork(updatedWork), updatedImageKey]]),
      updatedSyncedAt,
    )

    expect(await listDeviceSnapshots(db, 'device-1')).toEqual([
      {
        workId: 'work-1',
        title: 'Beta',
        imageKey: updatedImageKey,
        thumbnailWidth: 640,
        thumbnailHeight: 360,
        erogamescapeId: 123,
        officialUrl: 'https://example.com/official',
        erogamescapeUrl: 'https://example.com/egs',
        seiyaUrl: 'https://example.com/seiya',
      },
    ])
    expect(await findDeviceById(db, 'device-1')).toEqual({
      deviceId: 'device-1',
      secretHash: 'secret-hash-1',
      lastSyncedAt: updatedSyncedAt,
    })
  })
})
