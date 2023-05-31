import "virtual:uno.css";
import "@unocss/reset/tailwind-compat.css";
import "./index.css";
import App from "./App.svelte";

const app = new App({
  // @ts-expect-error
  target: document.getElementById("app"),
});

export default app;
