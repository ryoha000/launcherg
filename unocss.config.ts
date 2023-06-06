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
        success: "#347d39",
        edit: "#116329",
        error: "#EA4E60",
      },
      bg: {
        primary: "#22272e",
        secondary: "#2d333b",
        tertiary: "#323942",
        disabled: "#181818",
        button: "#373e47",
        buttonHover: "#444c56",
        backdrop: "#1C2128",
        successHover: "#46954a",
      },
      ui: { tertiary: "#636e7b" },
      border: {
        primary: "#444c56",
        button: "#CDD9E5",
        buttonHover: "#768390",
      },
      text: {
        primary: "#adbac7",
        secondary: "#CDD9E5",
        tertiary: "#768390",
        link: "#2e7cd5",
        white: "#FFFFFF",
      },
    },
    fontSize: {
      body: ["1rem", "160%"],
      body2: [".875rem", "160%"],
      body3: [".8rem", "160%"],
      h1: ["1.75rem", "145%"],
      h2: ["1.5rem", "145%"],
      h3: ["1.25rem", "145%"],
      h4: ["1.125rem", "145%"],
      caption: [".75rem", "142%"],
      input: [".875rem", "100%"],
    },
  },
});
