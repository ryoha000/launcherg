import type { Collection, CollectionElement } from "@/lib/types";
import { invoke } from "@tauri-apps/api/tauri";

export const commandGetAllCollections = async () => {
  return await invoke<Collection[]>("get_all_collections");
};

export const commandGetCollectionElements = async (id: number) => {
  return await invoke<CollectionElement[]>("get_collection_elements", { id });
};

export const addCollectionElementsInPc = async (
  exploreDirPaths: string[],
  useCache: boolean,
  addingCollectionId: string | null
) => {
  await invoke("add_collection_elements_in_pc", {
    exploreDirPaths,
    useCache,
    addingCollectionId,
  });
};

export const commandGetNearestKeyAndDistance = async (
  key: string,
  calculateDistanceKv: [string, string][]
) => {
  return await invoke<[string, number]>("get_nearest_key_and_distance", {
    key,
    calculateDistanceKv,
  });
};
