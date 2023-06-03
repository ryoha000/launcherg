import { localStorageWritable } from "@/lib/utils";

export type Tab = {
  id: number;
  type: "works" | "memos";
  scrollTo: number;
  title: string;
};
const createTabs = () => {
  return localStorageWritable<Tab[]>("tabs", [
    { id: 0, type: "works", scrollTo: 0, title: "G線上の魔王" },
    { id: 2, type: "memos", scrollTo: 0, title: "メモ - G線上の魔王" },
    { id: 3, type: "works", scrollTo: 0, title: "G線上の魔王" },
    { id: 4, type: "memos", scrollTo: 0, title: "メモ - G線上の魔王" },
  ]);
};

export const tabs = createTabs();
