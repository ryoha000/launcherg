import { writable } from "svelte/store";

export function createWritable<T>(initialValue: T) {
  let _value = initialValue;
  const store = writable<T>(initialValue);
  store.subscribe((v) => {
    _value = v;
  });
  return [store, () => _value] as const;
}
