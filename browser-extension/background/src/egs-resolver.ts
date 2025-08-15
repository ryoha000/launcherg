import type { EgsInfo } from '@launcherg/shared/proto/extension_internal'
import { create } from '@bufbuild/protobuf'
import { EgsInfoSchema } from '@launcherg/shared/proto/extension_internal'
import { createCache } from './kv-cache/index'

interface DmmKey { storeId: string, category: string, subcategory: string }
interface DlsiteKey { storeId: string, category: string }

function dmmKeyToString(key: DmmKey): string {
  return [key.storeId, key.category, key.subcategory].map(encodeURIComponent).join('|')
}

function dlsiteKeyToString(key: DlsiteKey): string {
  return [key.storeId, key.category].map(encodeURIComponent).join('|')
}

const dmmCache = createCache<EgsInfo>('egs:dmm')
const dlsiteCache = createCache<EgsInfo>('egs:dlsite')

export async function resolveEgsForDmm(storeId: string, category: string, subcategory: string): Promise<EgsInfo | null> {
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

export async function resolveEgsForDlsite(storeId: string, category: string): Promise<EgsInfo | null> {
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

async function runEgsQuery(query: string, colNums: number): Promise<string[] | null> {
  try {
    const form = new FormData()
    form.append('sql', query)
    const res = await fetch('https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php', {
      method: 'POST',
      body: form,
    })
    const text = await res.text()
    // 軽量パース（最初の結果行のみ）
    const rows = parseHtmlTable(text, colNums)
    return rows[0] ?? null
  }
  catch (e) {
    console.warn('EGS query failed', e)
    return null
  }
}

function parseHtmlTable(html: string, colNums: number): string[][] {
  const rows: string[][] = []
  const tableStart = html.indexOf('<table')
  if (tableStart === -1)
    return rows
  // 粗いが1行目ヘッダを飛ばし、2行目のみ抽出
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
