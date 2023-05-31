import { defineConfig } from "@unocss/vite";
import presetWind from "@unocss/preset-wind";
import presetAttributify from "@unocss/preset-attributify";
import presetWebFonts from "@unocss/preset-web-fonts";
import presetIcons from "@unocss/preset-icons";
import transformerVariantGroup from "@unocss/transformer-variant-group";
import extractorSvelte from "@unocss/extractor-svelte";

export default defineConfig({
  presets: [
    presetAttributify(),
    presetWind(),
    presetIcons(),
    extractorSvelte(),
    presetWebFonts({
      fonts: {
        sans: [
          {
            name: "Noto Sans JP",
            weights: ["400", "500", "700"],
          },
        ],
      },
    }),
  ],
  transformers: [transformerVariantGroup()],
  theme: {
    colors: {
      accent: {
        accent: "#487AF9",
        success: "#5EB917",
        error: "#EA4E60",
      },
      bg: {
        primary: "#22272e",
        secondary: "#2d333b",
        tertiary: "#323942",
        button: "#373e47",
        buttonHover: "#444c56",
      },
      border: {
        primary: "#444c56",
        button: "#CDD9E5",
        buttonHover: "#768390",
      },
      text: {
        primary: "#adbac7",
        placeholder: "#636e7b",
        link: "#2e7cd5",
      },
    },
    fontSize: {
      body: ["1rem", "160%"],
      body2: [".875rem", "160%"],
      h1: ["1.75rem", "145%"],
      h2: ["1.5rem", "145%"],
      h3: ["1.25rem", "145%"],
      h4: ["1.125rem", "145%"],
      caption: [".75rem", "142%"],
    },
  },
});
