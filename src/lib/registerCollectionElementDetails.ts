import {
  commandGetCollectionIdsByErogamescapeIds,
  commandGetNotRegisteredDetailErogamescapeIds,
  commandUpsertCollectionElementDetails,
} from '@/lib/command'
import { scrapeSql } from '@/lib/scrape/scrapeSql'

export async function registerCollectionElementDetails() {
  // 詳細未登録の EGS ID 群を取得
  const erogamescapeIds = await commandGetNotRegisteredDetailErogamescapeIds()
  if (!erogamescapeIds.length) {
    return
  }

  const query = `select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where gamelist.id IN (${erogamescapeIds.join(
    ', ',
  )});`
  const rows = await scrapeSql(query, 7)
  // EGS ID -> collection_element_id の解決
  const pairs = await commandGetCollectionIdsByErogamescapeIds(erogamescapeIds)
  const egsToCollectionId = new Map<number, number>(pairs)
  await commandUpsertCollectionElementDetails(
    rows.map(row => ({
      // row[0] は EGS ID。対応する collection_element_id を使う
      collectionElementId: egsToCollectionId.get(+row[0])!,
      gamename: row[1],
      gamenameRuby: row[2],
      sellday: row[3],
      isNukige: row[4].includes('t'),
      brandname: row[5],
      brandnameRuby: row[6],
    })),
  )
}
