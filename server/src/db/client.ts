import { drizzle } from 'drizzle-orm/d1'

import * as schema from '@server/db/schema'

export function createDbClient(database: D1Database) {
  return drizzle(database, { schema })
}

export type DbClient = ReturnType<typeof createDbClient>
