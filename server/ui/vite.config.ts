import { fileURLToPath } from 'url'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  root: fileURLToPath(new URL('.', import.meta.url).toString()),
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@ui': fileURLToPath(new URL('./src', import.meta.url).toString()),
      '@server': fileURLToPath(new URL('../src', import.meta.url).toString()),
    },
  },
  build: {
    outDir: fileURLToPath(new URL('./dist', import.meta.url).toString()),
    emptyOutDir: true,
  },
})
