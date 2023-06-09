import { commandGetAllElements } from "@/lib/command";
import type { CollectionElement } from "@/lib/types";
import { createWritable } from "@/lib/utils";

function createSidebarCollectionElements() {
  const [{ subscribe, set }, value] = createWritable<CollectionElement[]>([]);

  const refetch = async () => {
    set(await commandGetAllElements());
  };

  return {
    subscribe,
    value,
    refetch,
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
