import { commandGetAllElements } from "@/lib/command";
import type { CollectionElement } from "@/lib/types";
import { createWritable } from "@/lib/utils";

function createSidebarCollectionElements() {
  const [{ subscribe, update, set }, value] = createWritable<
    CollectionElement[]
  >([]);

  const refetch = async () => {
    set(await commandGetAllElements());
  };
  const updateLike = (id: number, isLike: boolean) => {
    update((elements) =>
      elements.map((v) =>
        v.id === id ? { ...v, likeAt: isLike ? "2023-06-09" : null } : { ...v }
      )
    );
  };

  return {
    subscribe,
    value,
    refetch,
    updateLike,
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
