import "virtual:uno.css";
import "@unocss/reset/tailwind-compat.css";
import "./index.scss";
import "tippy.js/dist/tippy.css";
import "simplebar";
import "simplebar/dist/simplebar.min.css";
import "easymde/dist/easymde.min.css";
import "./toast.scss";
import App from "./App.svelte";
import { mount } from "svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
