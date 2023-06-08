import type {
  Collection,
  CollectionElement,
  CollectionElementDetail,
} from "@/lib/types";
import { invoke } from "@tauri-apps/api/tauri";

export const commandGetAllCollections = async () => {
  return await invoke<Collection[]>("get_all_collections");
};

export const commandGetCollectionElements = async (id: number) => {
  return await invoke<CollectionElement[]>("get_collection_elements", { id });
};

export const commandAddCollectionElementsInPc = async (
  exploreDirPaths: string[],
  useCache: boolean,
  addingCollectionId: number | null
) => {
  return await invoke<string[]>("add_collection_elements_in_pc", {
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

export const commandGetDefaultImportDirs = async () => {
  return await invoke<string[]>("get_default_import_dirs", {});
};

export const commandPlayGame = async (
  collectionElementId: number,
  isRunAsAdmin: boolean
) => {
  return await invoke<void>("play_game", { collectionElementId, isRunAsAdmin });
};

export const commandGetPlayTomeMinutes = async (
  collectionElementId: number
) => {
  return await invoke<number>("get_play_time_minutes", { collectionElementId });
};

export const commandGetCollectionElement = async (
  collectionElementId: number
) => {
  return await invoke<CollectionElement>("get_collection_element", {
    collectionElementId,
  });
};

export const commandDeleteCollectionElement = async (
  collectionElementId: number
) => {
  return await invoke<void>("delete_collection_element", {
    collectionElementId,
  });
};

export const commandGetNotRegisterdDetailElementIds = async () => {
  return await invoke<number[]>("get_not_registered_detail_element_ids", {});
};

export const commandCreateElementDetails = async (
  details: CollectionElementDetail[]
) => {
  return await invoke<void>("create_element_details", {
    details,
  });
};

export const commandGetBrandnameAndRubies = async () => {
  return await invoke<[string, string][]>("get_brandname_and_rubies", {});
};

export const commandAddCollectionElementsByOption = async (
  collectionId: number,
  isNukige: boolean,
  notNukige: boolean,
  isExistPath: boolean,
  brandnames: string[] | null,
  between: [string, string] | null
) => {
  return await invoke<void>("add_collection_elements_by_option", {
    collectionId,
    isNukige,
    notNukige,
    isExistPath,
    brandnames,
    between,
  });
};
