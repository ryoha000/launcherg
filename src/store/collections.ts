import type { Collection } from "@/lib/types";
import { writable } from "svelte/store";

function createCollections() {
  const { subscribe, set, update } = writable<Collection[]>([]);

  const init = async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
    set(
      Array.from({ length: 5 }).map((_, i) => ({
        id: `collection-${i}`,
        name: `collection-${i}`,
        elements: Array.from({ length: 5 }).map((_, i) => ({
          id: Math.floor(Math.random() * 30000),
          gamename: `collection-element-${i}`,
          path: "",
          iconPath: "",
        })),
      }))
    );
  };

  return {
    subscribe,
    init,
  };
}

export const collections = createCollections();
