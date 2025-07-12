import { createLocalStorageWritable, localStorageWritable } from "@/lib/utils";
import { goto } from "@mateothegreat/svelte5-router";
import type { Hook } from "@mateothegreat/svelte5-router/hooks";

export type Tab = {
  id: number;
  workId: number;
  type: "works" | "memos";
  scrollTo: number;
  title: string;
};

const isValidTabType = (src: string): src is "works" | "memos" => {
  return src === "works" || src === "memos";
};

const extractId = (path: string) => {
  const match = path.match(/^\/(works|memos)\/(\d+)$/);
  if (match) {
    return Number(match[2]); // match[2] にID（12345など）が入っている
  } else {
    return null; // マッチしなければ null を返す
  }
};

const createTabs = () => {
  const [tabs, getTabs] = createLocalStorageWritable<Tab[]>("tabs", [
    { id: 0, workId: 7402, type: "works", scrollTo: 0, title: "G線上の魔王" },
    {
      id: 2,
      workId: 21228,
      type: "memos",
      scrollTo: 0,
      title: "メモ - G線上の魔王",
    },
    { id: 3, workId: 20460, type: "works", scrollTo: 0, title: "G線上の魔王" },
    {
      id: 4,
      workId: 21531,
      type: "memos",
      scrollTo: 0,
      title: "メモ - G線上の魔王",
    },
  ]);

  const [selected, getSelected] = createLocalStorageWritable("tab-selected", 0);

  const routeLoaded: Hook = (event) => {
    console.log("routeLoaded", event);
    const path = event.result.path.original;
    const isHome = path === "/";
    if (isHome) {
      selected.set(-1);
      return;
    }

    const id = extractId(path);
    if (!id || isNaN(id)) {
      console.error("params[id] is undefined (not home)");
      return;
    }

    const tabType = event.result.path.original.split("/")[1];
    if (!isValidTabType(tabType)) {
      console.error("tabType is invalid (not home)");
      return;
    }

    const tabIndex = getTabs().findIndex(
      (v) => v.workId === id && v.type === tabType
    );
    if (tabIndex === -1) {
      const searchParams = new URLSearchParams(event.result.querystring.params);
      const gamename = searchParams.get("gamename");
      if (!gamename) {
        console.error("tabs にないのに gamename の queryParam がない");
        return;
      }
      let title = gamename;
      if (tabType === "memos") {
        title = `メモ - ${title}`;
      }
      const newTab: Tab = {
        id: new Date().getTime(),
        type: tabType,
        workId: id,
        scrollTo: 0,
        title,
      };
      tabs.update((v) => {
        return [...v, newTab];
      });
      const newSelected = getTabs().length - 1;
      selected.set(newSelected);
    } else {
      selected.set(tabIndex);
    }

    return true;
  };
  const deleteTab = (id: number) => {
    const deleteIndex = getTabs().findIndex((v) => v.id === id);
    const currentIndex = getSelected();

    const isCurrentTab = deleteIndex === currentIndex;
    const isDeletePrevTab = deleteIndex < currentIndex;
    const isRightestTab = deleteIndex === getTabs().length - 1;

    tabs.update((v) => {
      const newTabs = v.filter((tab) => tab.id !== id);
      if (newTabs.length === 0) {
        goto("/");
      }
      return newTabs;
    });

    if (isRightestTab && getTabs().length === 0) {
      // すでに home へ遷移済
      return;
    }

    if (isCurrentTab) {
      const newIndex = isRightestTab ? currentIndex - 1 : currentIndex;
      const nextTab = getTabs()[newIndex];
      goto(`/${nextTab.type}/${nextTab.workId}`);
      return;
    }

    if (isDeletePrevTab) {
      selected.update((v) => v - 1);
      return;
    }
  };
  const initialize = () => {
    const _tabs = getTabs();
    const index = getSelected();
    if (_tabs.length - 1 < index) {
      console.error("_tabs.length - 1 < index", {
        tabs: getTabs(),
        selected: getSelected(),
      });
      selected.set(-1);
      goto("/");
      return;
    }
    if (index < 0) {
      goto("/");
      return;
    }
    const tab = _tabs[index];
    goto(`/${tab.type}/${tab.workId}`);
  };
  const getSelectedTab = () => getTabs()[getSelected()];
  return {
    tabs,
    selected: {
      subscribe: selected.subscribe,
    },
    getSelectedTab,
    routeLoaded,
    deleteTab,
    initialize,
  };
};

export const {
  tabs,
  selected,
  getSelectedTab,
  routeLoaded,
  deleteTab,
  initialize,
} = createTabs();
