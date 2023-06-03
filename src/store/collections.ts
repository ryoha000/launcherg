import { commandGetAllCollections } from "@/lib/command";
import type { Collection } from "@/lib/types";
import { writable } from "svelte/store";

function createCollections() {
  const { subscribe, set, update } = writable<Collection[]>([]);

  const init = async () => {
    const res = await commandGetAllCollections();
    set(res);
  };

  return {
    subscribe,
    init,
  };
}

export const collections = createCollections();
