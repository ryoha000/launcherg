import { getWorkByScrape } from "@/lib/scrapeWork";
import type { Work } from "@/lib/types";
import { createLocalStorageWritable } from "@/lib/utils";

const createWorks = () => {
  const [cache, getCache] = createLocalStorageWritable<Record<number, Work>>(
    "works-cache",
    {}
  );

  const get = async (id: number) => {
    const cachedWork = getCache()[id];
    if (cachedWork) {
      return cachedWork;
    }
    const work = await getWorkByScrape(id);
    cache.update((v) => {
      v[id] = work;
      return v;
    });
    return work;
  };

  return {
    subscribe: cache.subscribe,
    get,
  };
};

export const works = createWorks();
