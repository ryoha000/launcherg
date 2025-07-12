import Home from "@/views/Home.svelte";
import Memo from "@/views/Memo.svelte";
import Work from "@/views/Work.svelte";
import type { RouteConfig } from "@mateothegreat/svelte5-router/route.svelte";

export const routes: RouteConfig[] = [
  { path: "/", component: Home },
  { path: "/works/:id", component: Work },
  { path: "/memos/:id", component: Memo },
  // TODO: 404
];
