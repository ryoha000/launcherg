import type { Collection } from "@/lib/types";
import { invoke } from "@tauri-apps/api/tauri";

export const commandGetAllCollections = async () => {
  return await invoke<Collection[]>("get_all_collections");
};

export const addCollectionElementsInPc = async (
  exploreDirPaths: string[],
  withCache: boolean,
  addingCollectionId: string | null
) => {
  await invoke("add_collection_elements_in_pc", {
    exploreDirPaths,
    withCache,
    addingCollectionId,
  });
};
