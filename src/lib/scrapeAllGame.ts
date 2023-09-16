import {
  commandGetAllGameCacheLastUpdated,
  commandUpdateAllGameCache,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";
import type { AllGameCacheOne } from "@/lib/types";
import { ResponseType, fetch } from "@tauri-apps/api/http";

const STEP = 5000;
const MAX_SCRAPE_COUNT = 20;

export const scrapeAllGame = async (idCursor = 0) => {
  const idGameNamePairs: AllGameCacheOne[] = [];
  for (let i = 0; i < MAX_SCRAPE_COUNT; i++) {
    const query = `SELECT id, gamename, CASE WHEN gamelist.dmm_genre='digital' AND gamelist.dmm_genre_2='pcgame' THEN 'http://pics.dmm.co.jp/digital/pcgame/' || gamelist.dmm || '/' || gamelist.dmm || 'pl.jpg'
    WHEN gamelist.dmm_genre='digital' AND gamelist.dmm_genre_2='doujin' THEN 'https://doujin-assets.dmm.co.jp/digital/game/' || gamelist.dmm || '/' || gamelist.dmm || 'pr.jpg'
    WHEN gamelist.dlsite_id IS NOT NULL AND (gamelist.dlsite_domain='pro' OR gamelist.dlsite_domain='soft') THEN 'https://img.dlsite.jp/modpub/images2/work/professional/' || left(gamelist.dlsite_id,2) || CAST(right(left(gamelist.dlsite_id,5),3) AS INTEGER)+1 || '000/' || gamelist.dlsite_id || '_img_main.jpg'
    WHEN gamelist.dlsite_id IS NOT NULL THEN 'https://img.dlsite.jp/modpub/images2/work/doujin/' || left(gamelist.dlsite_id,2) || CAST(right(left(gamelist.dlsite_id,5),3) AS INTEGER)+1 || '000/' || gamelist.dlsite_id || '_img_main.jpg'
    ELSE 'https://pics.dmm.co.jp/mono/game/' || gamelist.dmm || '/' || gamelist.dmm || 'pl.jpg' END AS thumbnail_url FROM gamelist WHERE id >= ${idCursor} AND id < ${
      idCursor + STEP
    } AND model = 'PC';`;
    const rows = await scrapeSql(query, 3);
    if (!rows.length) {
      console.log(
        `end within ${i + 1} loop. games.length: ${idGameNamePairs.length}`
      );
      break;
    }
    idGameNamePairs.push(
      ...rows.map((v) => ({ id: +v[0], gamename: v[1], thumbnailUrl: v[2] }))
    );
    await new Promise((resolve) => setTimeout(resolve, 2000));
    idCursor += STEP;
  }

  return idGameNamePairs;
};

export const initializeAllGameCache = async () => {
  let objValue: AllGameCacheOne[] = [];
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
      await fetch<AllGameCacheOne[]>(
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
  await commandUpdateAllGameCache(objValue);
};
