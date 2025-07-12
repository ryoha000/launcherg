import extractorSvelte from '@unocss/extractor-svelte'
import presetAttributify from '@unocss/preset-attributify'
import presetIcons from '@unocss/preset-icons'
import presetWebFonts from '@unocss/preset-web-fonts'
import presetWind from '@unocss/preset-wind'
import transformerVariantGroup from '@unocss/transformer-variant-group'
import { defineConfig } from '@unocss/vite'

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
            name: 'Noto Sans JP',
            weights: ['400', '500', '700'],
          },
        ],
        logo: [
          {
            name: 'Space Mono',
            weights: ['400'],
          },
        ],
      },
    }),
  ],
  transformers: [transformerVariantGroup()],
  theme: {
    colors: {
      accent: {
        accent: '#487AF9',
        success: '#347d39',
        edit: '#116329',
        warning: '#c69026',
        error: '#EA4E60',
      },
      bg: {
        primary: '#22272e',
        secondary: '#2d333b',
        tertiary: '#323942',
        disabled: '#181818',
        button: '#373e47',
        buttonHover: '#444c56',
        backdrop: '#1C2128',
        successDisabled: 'rgba(35,134,54,0.6)',
        successHover: '#46954a',
        warning: '#37332a',
      },
      ui: { tertiary: '#636e7b' },
      border: {
        primary: '#444c56',
        button: '#CDD9E5',
        buttonHover: '#768390',
        warning: '#AE7C14',
        successDisabled: 'rgba(35,134,54,0.6)',
      },
      text: {
        primary: '#adbac7',
        secondary: '#CDD9E5',
        tertiary: '#768390',
        link: '#2e7cd5',
        white: '#FFFFFF',
        disabled: '#484f58',
        successDisabled: 'rgba(255,255,255,0.5)',
      },
    },
    fontSize: {
      body: ['1rem', '160%'],
      body2: ['.875rem', '160%'],
      body3: ['.8rem', '160%'],
      h1: ['1.75rem', '145%'],
      h2: ['1.5rem', '145%'],
      h3: ['1.25rem', '145%'],
      h4: ['1.125rem', '145%'],
      caption: ['.75rem', '142%'],
      input: ['.875rem', '100%'],
    },
  },
})
