import {
  commandCreateElementDetails,
  commandGetNotRegisterdDetailElementIds,
} from '@/lib/command'
import { scrapeSql } from '@/lib/scrape/scrapeSql'

export async function registerCollectionElementDetails() {
  const ids = await commandGetNotRegisterdDetailElementIds()
  if (!ids.length) {
    return
  }

  const query = `select gamelist.id, gamelist.gamename, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where gamelist.id IN (${ids.join(
    ', ',
  )});`
  const rows = await scrapeSql(query, 7)
  await commandCreateElementDetails(
    rows.map(row => ({
      collectionElementId: +row[0],
      gamename: row[1],
      gamenameRuby: row[2],
      sellday: row[3],
      isNukige: row[4].includes('t'),
      brandname: row[5],
      brandnameRuby: row[6],
    })),
  )
}
