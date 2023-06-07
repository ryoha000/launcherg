import { writable } from "svelte/store";

export function createWritable<T>(initialValue: T) {
  let _value = initialValue;
  const store = writable<T>(initialValue);
  store.subscribe((v) => {
    _value = v;
  });
  return [store, () => _value] as const;
}

export const localStorageWritable = <T>(key: string, initialValue: T) => {
  let stored = localStorage.getItem(key);
  const store = writable<T>(stored ? JSON.parse(stored) : initialValue);
  store.subscribe((value) => localStorage.setItem(key, JSON.stringify(value)));
  return store;
};

export const createLocalStorageWritable = <T>(key: string, initialValue: T) => {
  let _value = initialValue;
  const store = localStorageWritable<T>(key, initialValue);
  store.subscribe((v) => {
    _value = v;
  });
  return [store, () => _value] as const;
};

export type Cache<S extends string | number, U> = Record<
  S,
  { createdAt: number; value: U }
>;

export const createLocalStorageCache = <K extends string | number, T>(
  key: string,
  fetcher: (key: K) => Promise<T>,
  invalidateMilliseconds = 1000 * 60 * 60 * 24
) => {
  const initialValue = {} as Cache<K, T>;
  const [cache, getCache] = createLocalStorageWritable(key, initialValue);

  const getter = async (key: K): Promise<T> => {
    const now = new Date().getTime();
    const cached = getCache()[key];
    if (cached && now < cached.createdAt + invalidateMilliseconds) {
      return cached.value;
    }
    const value = await fetcher(key);
    cache.update((v) => {
      v[key] = { value: value, createdAt: now };
      return v;
    });
    return value;
  };
  return getter;
};

export const convertSpecialCharacters = (str: string) => {
  var tempElement = document.createElement("textarea");
  tempElement.innerHTML = str;
  return tempElement.value;
};
