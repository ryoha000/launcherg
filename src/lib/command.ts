import type { Collection } from "@/lib/types";
import { invoke } from "@tauri-apps/api/tauri";

export const commandGetAllCollections = async () => {
  return await invoke<Collection[]>("get_all_collections");
};
