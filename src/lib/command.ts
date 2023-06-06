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

export const commandUploadImage = async (id: number, base64Image: string) => {
  return await invoke<string>("upload_image", {
    id,
    base64Image,
  });
};

export const commandGetMemoPath = async (id: number) => {
  return await invoke<string>("get_memo_path", {
    id,
  });
};

export const commandCreateNewCollection = async (name: string) => {
  return await invoke<Collection>("create_new_collection", { name });
};

export const commandUpdateCollection = async (id: number, name: string) => {
  return await invoke<void>("update_collection", { id, name });
};

export const commandDeleteCollection = async (id: number) => {
  return await invoke<void>("delete_collection", { id });
};

export const commandUpsertCollectionElement = async (
  id: number,
  gamename: string,
  path: string
) => {
  return await invoke<void>("upsert_collection_element", {
    id,
    gamename,
    path,
  });
};

export const commandUpdateCollectionElementIcon = async (
  id: number,
  path: string
) => {
  return await invoke<void>("update_collection_element_icon", {
    id,
    path,
  });
};

export const commandAddElementsToCollection = async (
  collectionId: number,
  collectionElementIds: number[]
) => {
  return await invoke<void>("add_elements_to_collection", {
    collectionId,
    collectionElementIds,
  });
};

export const commandRemoveElementsFromCollection = async (
  collectionId: number,
  collectionElementIds: number[]
) => {
  return await invoke<void>("remove_elements_from_collection", {
    collectionId,
    collectionElementIds,
  });
};
