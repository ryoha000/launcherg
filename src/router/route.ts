import Home from "@/views/Home.svelte";
import Work from "@/views/Work.svelte";

export const routes = {
  "/": Home,
  "/works/:id": Work,
  // TODO: memos
  // TODO: 404
};
