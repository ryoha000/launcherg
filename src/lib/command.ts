import type {
  AllGameCacheOne,
  CollectionElement,
  CollectionElementDetail,
} from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";

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

export const commandUpsertCollectionElement = async (arg: {
  exePath: string | null;
  lnkPath: string | null;
  gameCache: AllGameCacheOne;
}) => {
  return await invoke<void>("upsert_collection_element", arg);
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
  return await invoke<number | null>("play_game", {
    collectionElementId,
    isRunAsAdmin,
  });
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

export const commandGetAllGameCacheLastUpdated = async () => {
  const [id, dateString] = await invoke<[number, string]>(
    "get_all_game_cache_last_updated"
  );
  return { id, date: new Date(dateString) };
};

export const commandUpdateAllGameCache = async (
  gameCaches: AllGameCacheOne[]
) => {
  await invoke("update_all_game_cache", {
    gameCaches,
  });
};

export const commandGetGameCandidates = async (filepath: string) => {
  return await invoke<[number, string][]>("get_game_candidates", {
    filepath,
  });
};

export const commandGetExePathByLnk = async (filepath: string) => {
  return await invoke<string>("get_exe_path_by_lnk", {
    filepath,
  });
};

export const commandGetGameCacheById = async (id: number) => {
  return await invoke<AllGameCacheOne | null>("get_game_cache_by_id", {
    id,
  });
};

export const commandSaveScreenshotByPid = async (
  workId: number,
  processId: number
) => {
  return await invoke<string>("save_screenshot_by_pid", {
    workId,
    processId,
  });
};

export const commandUpdateCollectionElementThumbnails = async (
  ids: number[]
) => {
  return await invoke<void>("update_collection_element_thumbnails", {
    ids,
  });
};
