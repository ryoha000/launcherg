import Home from "@/views/Home.svelte";
import Memo from "@/views/Memo.svelte";
import Work from "@/views/Work.svelte";

export const routes = {
  "/": Home,
  "/works/:id": Work,
  "/memos/:id": Memo,
  // TODO: 404
};
