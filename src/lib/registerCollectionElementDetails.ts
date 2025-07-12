import {
  commandGetNotRegisterdDetailElementIds,
  commandCreateElementDetails,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrape/scrapeSql";

export const registerCollectionElementDetails = async () => {
  const ids = await commandGetNotRegisterdDetailElementIds();
  if (!ids.length) {
    return;
  }

  const query = `select gamelist.id, gamelist.furigana, gamelist.sellday, gamelist.okazu, brandlist.brandname, brandlist.brandfurigana from gamelist inner join brandlist on brandlist.id = gamelist.brandname where gamelist.id IN (${ids.join(
    ", "
  )});`;
  const rows = await scrapeSql(query, 6);
  await commandCreateElementDetails(
    rows.map((row) => ({
      collectionElementId: +row[0],
      gamenameRuby: row[1],
      sellday: row[2],
      isNukige: row[3].includes("t"),
      brandname: row[4],
      brandnameRuby: row[5],
    }))
  );
};
