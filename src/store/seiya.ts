import { commandGetNearestKeyAndDistance } from "@/lib/command";
import { getSeiyaDataPairs } from "@/lib/scrape/scrapeSeiya";
import type { SeiyaDataPair } from "@/lib/types";
import { createLocalStorageCache } from "@/lib/utils";

const createSeiya = () => {
  const getter = createLocalStorageCache<"master", SeiyaDataPair[]>(
    "seiya-cache",
    getSeiyaDataPairs
  );

  const getUrl = async (gamename: string) => {
    const cache = await getter("master");
    const [url, distance] = await commandGetNearestKeyAndDistance(
      gamename,
      cache
    );
    return url;
  };

  return {
    getUrl,
  };
};

export const seiya = createSeiya();
