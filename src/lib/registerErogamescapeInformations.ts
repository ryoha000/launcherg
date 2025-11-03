import { commandGetNotRegisteredErogamescapeInformationIds, commandUpsertErogamescapeInformation } from '@/lib/command'
import { scrapeSql } from '@/lib/scrape/scrapeSql'

export async function registerErogamescapeInformations() {
  // 詳細未登録の EGS 情報 ID 群を取得
  const erogamescapeIds = await commandGetNotRegisteredErogamescapeInformationIds()
  if (!erogamescapeIds.length) {
    return
  }

  const query = `select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where gamelist.id IN (${erogamescapeIds.join(
    ', ',
  )});`
  const rows = await scrapeSql(query, 7)
  await commandUpsertErogamescapeInformation(
    rows.map(row => ({
      erogamescapeId: +row[0],
      gamenameRuby: row[2],
      sellday: row[3],
      isNukige: row[4].includes('t'),
      brandname: row[5],
      brandnameRuby: row[6],
    })),
  )
}
