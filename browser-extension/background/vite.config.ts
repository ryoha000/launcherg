import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    emptyOutDir: true,
    outDir: '../dist/background',
    lib: {
      entry: resolve(__dirname, 'src/background.ts'),
      name: 'background',
      fileName: 'background',
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
