import type { EgsInfo } from '@launcherg/shared/proto/extension_internal'
import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { createKvCache } from '../kv-cache'

interface DmmKey { storeId: string, category: string, subcategory: string }
interface DlsiteKey { storeId: string, category: string }

function dmmKeyToString(key: DmmKey): string {
  return [key.storeId, key.category, key.subcategory].map(encodeURIComponent).join('|')
}

function dlsiteKeyToString(key: DlsiteKey): string {
  return [key.storeId, key.category].map(encodeURIComponent).join('|')
}

function createCaches() {
  const dmmCache = createKvCache<EgsInfo>('egs:dmm')
  const dlsiteCache = createKvCache<EgsInfo>('egs:dlsite')
  return { dmmCache, dlsiteCache }
}

export function createEgsResolver() {
  const { dmmCache, dlsiteCache } = createCaches()
  async function resolveForDmm(storeId: string, category: string, subcategory: string): Promise<EgsInfo | null> {
    const key: DmmKey = { storeId, category, subcategory }
    const cached = await dmmCache.get(dmmKeyToString(key))
    if (cached)
      return cached
    const query = `select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where dmm = '${escapeSql(storeId)}' and dmm_genre = '${escapeSql(category)}' and dmm_genre_2 = '${escapeSql(subcategory)}' limit 1;`
    const row = await runEgsQuery(query, 7)
    if (!row)
      return null
    const resolved: EgsInfo = create(EgsInfoSchema, {
      erogamescapeId: +row[0],
      gamename: row[1],
      gamenameRuby: row[2],
      sellday: row[3],
      isNukige: row[4]?.includes('t') ?? false,
      brandname: row[5],
      brandnameRuby: row[6],
    })
    await dmmCache.set(dmmKeyToString(key), resolved)
    return resolved
  }

  async function resolveForDlsite(storeId: string, category: string): Promise<EgsInfo | null> {
    const key: DlsiteKey = { storeId, category }
    const cached = await dlsiteCache.get(dlsiteKeyToString(key))
    if (cached)
      return cached
    const query = `select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where dlsite_id = '${escapeSql(storeId)}' and dlsite_domain = '${escapeSql(category)}' limit 1;`
    const row = await runEgsQuery(query, 7)
    if (!row)
      return null
    const resolved: EgsInfo = create(EgsInfoSchema, {
      erogamescapeId: +row[0],
      gamename: row[1],
      gamenameRuby: row[2],
      sellday: row[3],
      isNukige: row[4]?.includes('t') ?? false,
      brandname: row[5],
      brandnameRuby: row[6],
    })
    await dlsiteCache.set(dlsiteKeyToString(key), resolved)
    return resolved
  }

  async function resolveForDmmBulk(items: Array<{ storeId: string, category: string, subcategory: string }>): Promise<Array<EgsInfo | null>> {
    const results: Array<EgsInfo | null> = Array.from({ length: items.length }, () => null)
    const missingKeyToIndexes = new Map<string, number[]>()
    await Promise.all(items.map(async (item, index) => {
      const keyStr = dmmKeyToString({ storeId: item.storeId, category: item.category, subcategory: item.subcategory })
      const cached = await dmmCache.get(keyStr)
      if (cached) {
        results[index] = cached
        return
      }
      const bucket = missingKeyToIndexes.get(keyStr)
      if (bucket)
        bucket.push(index)
      else
        missingKeyToIndexes.set(keyStr, [index])
    }))
    if (missingKeyToIndexes.size === 0)
      return results
    const uniqueKeys = [...missingKeyToIndexes.keys()].map(k => k.split('|').map(decodeURIComponent) as [string, string, string])
    const buildTuples = (keys: Array<[string, string, string]>) => keys.map(([storeId, category, subcategory]) => `('${escapeSql(storeId)}','${escapeSql(category)}','${escapeSql(subcategory)}')`).join(', ')
    const CHUNK_SIZE = 200
    for (let start = 0; start < uniqueKeys.length; start += CHUNK_SIZE) {
      const chunk = uniqueKeys.slice(start, start + CHUNK_SIZE)
      const tuples = buildTuples(chunk)
      const query = `
select distinct on (gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2)
  gamelist.id,
  gamelist.gamename,
  gamelist.furigana,
  gamelist.sellday,
  gamelist.okazu,
  brandlist.brandname,
  brandlist.brandfurigana,
  gamelist.dmm,
  gamelist.dmm_genre,
  gamelist.dmm_genre_2
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where (gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2) in (${tuples})
order by gamelist.dmm, gamelist.dmm_genre, gamelist.dmm_genre_2, gamelist.id asc;`
      const rows = await runEgsQueryAll(query, 10)
      for (const row of rows) {
        const resolved: EgsInfo = create(EgsInfoSchema, {
          erogamescapeId: +row[0],
          gamename: row[1],
          gamenameRuby: row[2],
          sellday: row[3],
          isNukige: row[4]?.includes('t') ?? false,
          brandname: row[5],
          brandnameRuby: row[6],
        })
        const keyStr = [row[7], row[8], row[9]].map(encodeURIComponent).join('|')
        await dmmCache.set(keyStr, resolved)
      }
    }
    for (const [keyStr, idxs] of missingKeyToIndexes.entries()) {
      const cached = await dmmCache.get(keyStr)
      for (const idx of idxs)
        results[idx] = cached ?? null
    }
    return results
  }

  async function resolveForDlsiteBulk(items: Array<{ storeId: string, category: string }>): Promise<Array<EgsInfo | null>> {
    const results: Array<EgsInfo | null> = Array.from({ length: items.length }, () => null)
    const missingKeyToIndexes = new Map<string, number[]>()
    await Promise.all(items.map(async (item, index) => {
      const keyStr = dlsiteKeyToString({ storeId: item.storeId, category: item.category })
      const cached = await dlsiteCache.get(keyStr)
      if (cached) {
        results[index] = cached
        return
      }
      const bucket = missingKeyToIndexes.get(keyStr)
      if (bucket)
        bucket.push(index)
      else
        missingKeyToIndexes.set(keyStr, [index])
    }))
    if (missingKeyToIndexes.size === 0)
      return results
    const uniqueKeys = [...missingKeyToIndexes.keys()].map(k => k.split('|').map(decodeURIComponent) as [string, string])
    const buildTuples = (keys: Array<[string, string]>) => keys.map(([storeId, category]) => `('${escapeSql(storeId)}','${escapeSql(category)}')`).join(', ')
    const CHUNK_SIZE = 200
    for (let start = 0; start < uniqueKeys.length; start += CHUNK_SIZE) {
      const chunk = uniqueKeys.slice(start, start + CHUNK_SIZE)
      const tuples = buildTuples(chunk)
      const query = `
select distinct on (gamelist.dlsite_id, gamelist.dlsite_domain)
  gamelist.id,
  gamelist.gamename,
  gamelist.furigana,
  gamelist.sellday,
  gamelist.okazu,
  brandlist.brandname,
  brandlist.brandfurigana,
  gamelist.dlsite_id,
  gamelist.dlsite_domain
from gamelist
inner join brandlist on brandlist.id = gamelist.brandname
where (gamelist.dlsite_id, gamelist.dlsite_domain) in (${tuples})
order by gamelist.dlsite_id, gamelist.dlsite_domain, gamelist.id asc;`
      const rows = await runEgsQueryAll(query, 9)
      for (const row of rows) {
        const resolved: EgsInfo = create(EgsInfoSchema, {
          erogamescapeId: +row[0],
          gamename: row[1],
          gamenameRuby: row[2],
          sellday: row[3],
          isNukige: row[4]?.includes('t') ?? false,
          brandname: row[5],
          brandnameRuby: row[6],
        })
        const keyStr = [row[7], row[8]].map(encodeURIComponent).join('|')
        await dlsiteCache.set(keyStr, resolved)
      }
    }
    for (const [keyStr, idxs] of missingKeyToIndexes.entries()) {
      const cached = await dlsiteCache.get(keyStr)
      for (const idx of idxs)
        results[idx] = cached ?? null
    }
    return results
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
