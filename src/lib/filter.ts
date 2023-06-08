export type Option<T> = { label: string; value: T; otherLabels?: string[] };
import type { CollectionElement } from "@/lib/types";
import { isNotNullOrUndefined } from "@/lib/utils";
import { writable, type Readable } from "svelte/store";
import TrieSearch from "trie-search";
import { toHiragana, toRomaji } from "wanakana";

type KeyValue<T> = {
  key: string;
  value: T;
};

export const collectionElementsToOptions = (elements: CollectionElement[]) =>
  elements.map((v) => ({
    label: v.gamename,
    value: v.id,
    otherLabels: [
      toHiragana(v.gamenameRuby),
      toRomaji(v.gamenameRuby),
      v.brandname,
      toHiragana(v.brandnameRuby),
      toRomaji(v.brandnameRuby),
    ],
  }));

export const useFilter = <T>(
  options: Readable<Option<T>[]>,
  getOptions: () => Option<T>[]
) => {
  const query = writable("");
  const filtered = writable<Option<T>[]>([...getOptions()]);

  const init = () => {
    query.set("");
    const lazyQuery = writable("");
    filtered.set([...getOptions()]);

    const optionMap = new Map<Option<T>["value"], Option<T>>();
    for (const option of getOptions()) {
      optionMap.set(option.value, option);
    }

    const cache: Record<string, Option<T>[]> = {};

    let lazyQueryTimer: ReturnType<typeof setTimeout> | null = null;
    query.subscribe((_query) => {
      if (lazyQueryTimer) {
        clearTimeout(lazyQueryTimer);
      }
      lazyQueryTimer = setTimeout(() => {
        lazyQuery.set(_query.toLowerCase());
        lazyQueryTimer = null;
      }, 200);
    });
    lazyQuery.subscribe((_query) => {
      if (!_query) {
        return filtered.set([...getOptions()]);
      }
      const cached = Object.entries(cache).find(([input, _]) =>
        _query.includes(input)
      );
      const targetOptions = cached ? cached[1] : getOptions();
      const _filtered = targetOptions.filter((option) =>
        [option.label, ...(option.otherLabels ?? [])].find((key) =>
          key.toLowerCase().includes(_query)
        )
      );
      cache[_query] = _filtered;
      filtered.set(_filtered);
    });
  };
  init();

  options.subscribe(() => init());

  return { query, filtered };
};
