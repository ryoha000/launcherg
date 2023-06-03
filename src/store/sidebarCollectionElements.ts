import { commandGetCollectionElements } from "@/lib/command";
import type { CollectionElement } from "@/lib/types";
import { createWritable } from "@/lib/utils";

function createSidebarCollectionElements() {
  const [{ subscribe, set }, value] = createWritable<CollectionElement[]>([]);

  const init = async (id: number) => {
    const res = await commandGetCollectionElements(id);
    set(res);
  };

  return {
    subscribe,
    init,
    value,
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
