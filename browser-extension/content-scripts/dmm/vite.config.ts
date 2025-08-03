import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    emptyOutDir: false,
    outDir: '../../dist/content-scripts',
    lib: {
      entry: resolve(__dirname, 'src/dmm-extractor.ts'),
      name: 'dmm-extractor',
      fileName: 'dmm-extractor',
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
