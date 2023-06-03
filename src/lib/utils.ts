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
