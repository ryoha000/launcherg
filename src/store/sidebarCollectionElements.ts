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
    const now = new Date();
    const likeAt = `${now.getFullYear()}-${
      now.getMonth() + 1
    }-${now.getDate()}`;
    update((elements) =>
      elements.map((v) =>
        v.id === id ? { ...v, likeAt: isLike ? likeAt : null } : { ...v }
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
