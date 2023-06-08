import { commandGetCollectionElements } from "@/lib/command";
import type { CollectionElement } from "@/lib/types";
import { createWritable } from "@/lib/utils";

function createSidebarCollectionElements() {
  const [{ subscribe, set }, value] = createWritable<CollectionElement[]>([]);

  let _id = 0;
  const init = async (id: number) => {
    const res = await commandGetCollectionElements(id);
    console.log("init", res);
    set(res);
    _id = id;
  };

  const refetch = async () => {
    if (!_id) {
      return;
    }
    const res = await commandGetCollectionElements(_id);
    console.log("refetch", res);
    set(res);
  };

  return {
    subscribe,
    init,
    value,
    refetch,
  };
}

export const sidebarCollectionElements = createSidebarCollectionElements();
