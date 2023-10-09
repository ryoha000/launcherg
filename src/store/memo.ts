import { writable } from "svelte/store";

export const memo = writable<
  { workId: number; value: string; lastModified: "remote" | "local" }[]
>([]);
