import { scrapeSql } from "@/lib/scrapeSql";

const STEP = 5000;
const MAX_SCRAPE_COUNT = 20;

export const scrapeAllGame = async (idCursor = 0) => {
  const idGameNamePairs: { id: number; gamename: string }[] = [];
  for (let i = 0; i < MAX_SCRAPE_COUNT; i++) {
    const query = `SELECT id, gamename FROM gamelist WHERE id >= ${idCursor} AND id < ${
      idCursor + STEP
    } AND model = 'PC';`;
    const rows = await scrapeSql(query, 2);
    if (!rows.length) {
      console.log(
        `end within ${i + 1} loop. games.length: ${idGameNamePairs.length}`
      );
      break;
    }
    idGameNamePairs.push(...rows.map((v) => ({ id: +v[0], gamename: v[1] })));
    await new Promise((resolve) => setTimeout(resolve, 5000));
    idCursor += STEP;
  }

  console.log(JSON.stringify(idGameNamePairs));
};
