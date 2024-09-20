import {
  commandGetAllGameCacheLastUpdated,
  commandUpdateAllGameCache,
} from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";
import type { AllGameCacheOne } from "@/lib/types";
import { fetch } from "@tauri-apps/plugin-http";

const STEP = 5000;
const MAX_SCRAPE_COUNT = 20;

const ALL_GAME_CACHE_BASE_QUERY = `SELECT id, gamename, CASE WHEN dmm_genre='digital' AND dmm_genre_2='pcgame' THEN 'https://pics.dmm.co.jp/digital/pcgame/' || dmm || '/' || dmm || 'pl.jpg'
WHEN dmm_genre='digital' AND dmm_genre_2='doujin' THEN 'https://doujin-assets.dmm.co.jp/digital/game/' || dmm || '/' || dmm || 'pr.jpg'
WHEN dmm_genre='mono' AND dmm_genre_2='pcgame' THEN 'https://pics.dmm.co.jp/mono/game/' || dmm || '/' || dmm || 'pl.jpg'
WHEN dlsite_id IS NOT NULL AND (dlsite_domain='pro' OR dlsite_domain='soft') THEN 'https://img.dlsite.jp/modpub/images2/work/professional/' || left(dlsite_id,2) || LPAD(CAST(CAST(RIGHT(LEFT(dlsite_id, 5), 3) AS INTEGER) + 1 AS TEXT), 3, '0') || '000/' || dlsite_id || '_img_main.jpg'
WHEN dlsite_id IS NOT NULL THEN 'https://img.dlsite.jp/modpub/images2/work/doujin/' || left(dlsite_id,2) || LPAD(CAST(CAST(RIGHT(LEFT(dlsite_id, 5), 3) AS INTEGER) + 1 AS TEXT), 3, '0') || '000/' || dlsite_id || '_img_main.jpg'
WHEN dmm IS NOT NULL THEN 'https://pics.dmm.co.jp/mono/game/' || dmm || '/' || dmm || 'pl.jpg'
WHEN surugaya_1 IS NOT NULL THEN 'https://www.suruga-ya.jp/database/pics/game/' || surugaya_1 || '.jpg'
ELSE '' END AS thumbnail_url FROM gamelist`;

export const scrapeAllGameCacheOnes = async (ids: number[]) => {
  const idGameNamePairs: AllGameCacheOne[] = [];
  for (let i = 0; i < ids.length; i += STEP) {
    const inQuery = ids.map((v) => `'${v}'`).join(",");
    const query = `${ALL_GAME_CACHE_BASE_QUERY} WHERE id IN (${inQuery});`;
    const rows = await scrapeSql(query, 3);
    idGameNamePairs.push(
      ...rows.map((v) => ({ id: +v[0], gamename: v[1], thumbnailUrl: v[2] }))
    );
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }

  return idGameNamePairs;
};

export const scrapeAllGame = async (idCursor = 0) => {
  const idGameNamePairs: AllGameCacheOne[] = [];
  for (let i = 0; i < MAX_SCRAPE_COUNT; i++) {
    const query = `${ALL_GAME_CACHE_BASE_QUERY} WHERE id >= ${idCursor} AND id < ${
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
    const response = await fetch(
      "https://raw.githubusercontent.com/ryoha000/launcherg/main/script/all_games.json",
      { method: "GET" }
    );
    const initValue = (await response.json()) as AllGameCacheOne[];
    const maxId = initValue.reduce(
      (acc, cur) => (acc > cur.id ? acc : cur.id),
      0
    );
    objValue = [...initValue, ...(await scrapeAllGame(maxId + 1))];
  }
  await commandUpdateAllGameCache(objValue);
};
