import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    emptyOutDir: false,
    outDir: '../../dist/content-scripts',
    lib: {
      entry: {
        'dlsite-extractor': resolve(__dirname, 'src/main.ts'),
        'dlsite-network-hook': resolve(__dirname, 'src/network-hook.ts'),
      },
      formats: ['es'],
    },
    rollupOptions: {
      external: ['chrome'],
    },
    target: 'esnext',
    minify: false,
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
})
