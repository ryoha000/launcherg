import type {
  Collection,
  CollectionElement,
  CollectionElementDetail,
} from "@/lib/types";
import { invoke } from "@tauri-apps/api/tauri";

export const commandCreateElementsInPc = async (
  exploreDirPaths: string[],
  useCache: boolean
) => {
  return await invoke<string[]>("create_elements_in_pc", {
    exploreDirPaths,
    useCache,
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

export const commandGetAllElements = async () => {
  return await invoke<CollectionElement[]>("get_all_elements", {});
};

export const commandUpdateElementLike = async (id: number, isLike: boolean) => {
  return await invoke<void>("update_element_like", { id, isLike });
};

export const commandOpenFolder = async (path: string) => {
  return await invoke<void>("open_folder", { path });
};
