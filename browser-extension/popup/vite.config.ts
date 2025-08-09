import { copyFileSync, mkdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { defineConfig } from 'vite'

// プラグイン：popup.htmlとstyles.cssをコピー
function copyPopupAssetsPlugin() {
  return {
    name: 'copy-popup-assets',
    writeBundle() {
      // distディレクトリの作成を確実にする
      mkdirSync(resolve(__dirname, '../dist/popup'), { recursive: true })

      // popup.htmlをコピー
      copyFileSync(
        resolve(__dirname, 'src/popup.html'),
        resolve(__dirname, '../dist/popup/popup.html'),
      )

      // styles.cssをコピー
      copyFileSync(
        resolve(__dirname, 'src/styles.css'),
        resolve(__dirname, '../dist/popup/styles.css'),
      )
    },
  }
}

export default defineConfig({
  build: {
    emptyOutDir: false,
    outDir: '../dist/popup',
    lib: {
      entry: resolve(__dirname, 'src/popup.ts'),
      name: 'popup',
      fileName: 'popup',
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
  plugins: [copyPopupAssetsPlugin()],
})
