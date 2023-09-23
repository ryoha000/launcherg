import adapter from "@sveltejs/adapter-static"; // This was changed from adapter-auto
import { vitePreprocess } from "@sveltejs/kit/vite";
import UnoCSS from "@unocss/svelte-scoped/preprocess";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://kit.svelte.dev/docs/integrations#preprocessors
  // for more information about preprocessors
  preprocess: [vitePreprocess(), UnoCSS()],

  kit: {
    adapter: adapter(),
  },
};

export default config;
