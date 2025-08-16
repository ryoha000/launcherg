import type { EgsInfo } from '@launcherg/shared/proto/extension_internal'
import type { KvCache } from '../kv-cache'
import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { createKvCache } from '../kv-cache'

const BASE_COL_NUMS = 7
const BULK_CHUNK_SIZE = 200

const BASE_SELECT_COLUMNS = `
  gamelist.id,
  gamelist.gamename,
  gamelist.furigana,
  gamelist.sellday,
  gamelist.okazu,
  brandlist.brandname,
  brandlist.brandfurigana`

interface DmmKey { storeId: string, category: string, subcategory: string }
interface DlsiteKey { storeId: string, category: string }

function dmmKeyToString(key: DmmKey): string {
  return [key.storeId, key.category, key.subcategory].map(encodeURIComponent).join('|')
}

function dlsiteKeyToString(key: DlsiteKey): string {
  return [key.storeId, key.category].map(encodeURIComponent).join('|')
}

function splitKeyStr(keyStr: string): string[] {
  return keyStr.split('|').map(decodeURIComponent)
}

function buildTuples(keys: string[][]): string {
  return keys.map(arr => `(${arr.map(v => `'${escapeSql(v)}'`).join(', ')})`).join(', ')
}

function toEgsInfo(row: string[]): EgsInfo {
  return create(EgsInfoSchema, {
    erogamescapeId: +row[0],
    gamename: row[1],
    gamenameRuby: row[2],
    sellday: row[3],
    isNukige: row[4]?.includes('t') ?? false,
    brandname: row[5],
    brandnameRuby: row[6],
  })
}

function dmmKeyFromRow(row: string[]): string {
  return [row[7], row[8], row[9]].map(encodeURIComponent).join('|')
}

function dlsiteKeyFromRow(row: string[]): string {
  return [row[7], row[8]].map(encodeURIComponent).join('|')
}

function createCaches() {
  const dmmCache = createKvCache<EgsInfo>('egs:dmm')
  const dlsiteCache = createKvCache<EgsInfo>('egs:dlsite')
  return { dmmCache, dlsiteCache }
}

function createBulkCacheOps<TItem>(cache: KvCache<EgsInfo>, items: TItem[], deriveKey: (item: TItem) => string) {
  const results: Array<EgsInfo | null> = Array.from({ length: items.length }, () => null)

  // キャッシュヒットを results に反映し、未解決アイテムを返す
  const primeFromCache = async (): Promise<TItem[]> => {
    await Promise.all(items.map(async (item, index) => {
      const cached = await cache.get(deriveKey(item))
      if (cached)
        results[index] = cached
    }))
    return items.filter((_, index) => results[index] === null)
  }

  // フェッチ済みデータをキャッシュへ保存し、results にも反映
  const storeFetchedAndApply = async (data: { keyStr: string, value: EgsInfo }[]) => {
    await Promise.all(data.map(async (entry) => {
      await cache.set(entry.keyStr, entry.value)
    }))
    await Promise.all(items.map(async (item, index) => {
      const matched = data.find(entry => entry.keyStr === deriveKey(item))
      if (matched)
        results[index] = matched.value
    }))
    return items.filter((_, index) => results[index] === null)
  }

  return { results, primeFromCache, storeFetchedAndApply }
}

export function createEgsResolver() {
  const { dmmCache, dlsiteCache } = createCaches()
  async function resolveForDmm(storeId: string, category: string, subcategory: string): Promise<EgsInfo | null> {
    const key: DmmKey = { storeId, category, subcategory }
    const cached = await dmmCache.get(dmmKeyToString(key))
    if (cached)
      return cached
    const query = `select${BASE_SELECT_COLUMNS}
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where dmm = '${escapeSql(storeId)}' and dmm_genre = '${escapeSql(category)}' and dmm_genre_2 = '${escapeSql(subcategory)}'
limit 1;`
    const row = await runEgsQuery(query, BASE_COL_NUMS)
    if (!row)
      return null
    const resolved: EgsInfo = toEgsInfo(row)
    await dmmCache.set(dmmKeyToString(key), resolved)
    return resolved
  }

  async function resolveForDlsite(storeId: string, category: string): Promise<EgsInfo | null> {
    const key: DlsiteKey = { storeId, category }
    const cached = await dlsiteCache.get(dlsiteKeyToString(key))
    if (cached)
      return cached
    const query = `select${BASE_SELECT_COLUMNS}
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where dlsite_id = '${escapeSql(storeId)}' and dlsite_domain = '${escapeSql(category)}'
limit 1;`
    const row = await runEgsQuery(query, BASE_COL_NUMS)
    if (!row)
      return null
    const resolved: EgsInfo = toEgsInfo(row)
    await dlsiteCache.set(dlsiteKeyToString(key), resolved)
    return resolved
  }

  async function resolveForDmmBulk(items: Array<{ storeId: string, category: string, subcategory: string }>): Promise<Array<EgsInfo | null>> {
    const ops = createBulkCacheOps(dmmCache, items, dmmKeyToString)
    const missingItems = await ops.primeFromCache()
    if (missingItems.length === 0)
      return ops.results

    const uniqueKeyStrs = Array.from(new Set(missingItems.map(dmmKeyToString)))
    const uniqueKeys = uniqueKeyStrs.map(k => splitKeyStr(k))
    for (let start = 0; start < uniqueKeys.length; start += BULK_CHUNK_SIZE) {
      const chunk = uniqueKeys.slice(start, start + BULK_CHUNK_SIZE)
      const tuples = buildTuples(chunk)
      const query = `
select distinct on (gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2)
${BASE_SELECT_COLUMNS},
  gamelist.dmm,
  gamelist.dmm_genre,
  gamelist.dmm_genre_2
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where (gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2) in (${tuples})
order by gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2, gamelist.id asc;`
      const rows = await runEgsQueryAll(query, BASE_COL_NUMS + 3)
      const data = rows.map(row => ({ keyStr: dmmKeyFromRow(row), value: toEgsInfo(row) }))
      await ops.storeFetchedAndApply(data)
    }
    return ops.results
  }

  async function resolveForDlsiteBulk(items: Array<{ storeId: string, category: string }>): Promise<Array<EgsInfo | null>> {
    const ops = createBulkCacheOps(dlsiteCache, items, dlsiteKeyToString)
    const missingItems = await ops.primeFromCache()
    if (missingItems.length === 0)
      return ops.results

    const uniqueKeyStrs = Array.from(new Set(missingItems.map(dlsiteKeyToString)))
    const uniqueKeys = uniqueKeyStrs.map(k => splitKeyStr(k))
    for (let start = 0; start < uniqueKeys.length; start += BULK_CHUNK_SIZE) {
      const chunk = uniqueKeys.slice(start, start + BULK_CHUNK_SIZE)
      const tuples = buildTuples(chunk)
      const query = `
select distinct on (gamelist.dlsite_id, gamelist.dlsite_domain)
${BASE_SELECT_COLUMNS},
  gamelist.dlsite_id,
  gamelist.dlsite_domain
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where (gamelist.dlsite_id, gamelist.dlsite_domain) in (${tuples})
order by gamelist.dlsite_id, gamelist.dlsite_domain, gamelist.id asc;`
      const rows = await runEgsQueryAll(query, BASE_COL_NUMS + 2)
      const data = rows.map(row => ({ keyStr: dlsiteKeyFromRow(row), value: toEgsInfo(row) }))
      await ops.storeFetchedAndApply(data)
    }
    return ops.results
  }

  return { resolveForDmm, resolveForDlsite, resolveForDmmBulk, resolveForDlsiteBulk }
}

async function runEgsQuery(query: string, colNums: number): Promise<string[] | null> {
  try {
    const form = new FormData()
    form.append('sql', query)
    const res = await fetch('https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php', {
      method: 'POST',
      body: form,
    })
    const text = await res.text()
    const rows = parseHtmlTable(text, colNums)
    return rows[0] ?? null
  }
  catch (e) {
    console.warn('EGS query failed', e)
    return null
  }
}

async function runEgsQueryAll(query: string, colNums: number): Promise<string[][]> {
  try {
    const form = new FormData()
    form.append('sql', query)
    const res = await fetch('https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php', {
      method: 'POST',
      body: form,
    })
    const text = await res.text()
    return parseHtmlTable(text, colNums)
  }
  catch (e) {
    console.warn('EGS query failed', e)
    return []
  }
}

function parseHtmlTable(html: string, colNums: number): string[][] {
  const rows: string[][] = []
  const tableStart = html.indexOf('<table')
  if (tableStart === -1)
    return rows
  const trMatches = html.match(/<tr[\s\S]*?<\/tr>/g) || []
  for (let i = 1; i < trMatches.length; i++) {
    const tr = trMatches[i]
    const cols = [...tr.matchAll(/<td[^>]*>([\s\S]*?)<\/td>/g)].map(m => decodeHtml(m[1].trim()))
    if (cols.length >= colNums) {
      rows.push(cols.slice(0, colNums))
    }
  }
  return rows
}

function decodeHtml(s: string): string {
  return s
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, '\'')
}

function escapeSql(s: string): string {
  return s.replace(/'/g, '\'\'')
}
