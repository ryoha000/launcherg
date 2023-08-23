import {
  commandGetAllGameCacheLastUpdated,
  commandUpdateAllGameCache,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";
import { ResponseType, fetch, Body } from "@tauri-apps/api/http";

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
    await new Promise((resolve) => setTimeout(resolve, 3000));
    idCursor += STEP;
  }

  return idGameNamePairs;
};

export const initializeAllGameCache = async () => {
  let objValue: { id: number; gamename: string }[] = [];
  try {
    const lastUpdated = await commandGetAllGameCacheLastUpdated();
    const now = new Date();
    if (now.getTime() - lastUpdated.date.getTime() > 1000 * 60 * 60 * 24 * 1) {
      objValue = await scrapeAllGame(lastUpdated.id + 1);
    }
  } catch (e) {
    console.warn(
      "all_game_cache の取得に失敗しました。おそらく初期化されていないため初期化します。"
    );
    console.warn(e);
    const initValue = (
      await fetch<{ id: number; gamename: string }[]>(
        "https://raw.githubusercontent.com/ryoha000/launcherg/main/script/all_games.json",
        {
          method: "GET",
          responseType: ResponseType.JSON,
        }
      )
    ).data;
    const maxId = initValue.reduce(
      (acc, cur) => (acc > cur.id ? acc : cur.id),
      0
    );
    objValue = [...initValue, ...(await scrapeAllGame(maxId + 1))];
  }
  const value: [number, string][] = objValue.map((v) => [v.id, v.gamename]);
  await commandUpdateAllGameCache(value);
};
