import { getWorkByScrape } from "@/lib/scrape/scrapeWork";
import type { Work } from "@/lib/types";
import { createLocalStorageCache } from "@/lib/utils";

const createWorks = () => {
  const getter = createLocalStorageCache<number, Work>(
    "works-cache",
    getWorkByScrape
  );

  return {
    get: getter,
  };
};

export const works = createWorks();
