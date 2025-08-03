import { copyFileSync, mkdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { defineConfig } from 'vite'

// カスタムプラグイン：manifest.jsonとその他のアセットをコピー
function copyManifestPlugin() {
  return {
    name: 'copy-manifest',
    writeBundle() {
      // distディレクトリの作成を確実にする
      mkdirSync(resolve(__dirname, 'dist/popup'), { recursive: true })
      mkdirSync(resolve(__dirname, 'dist/config'), { recursive: true })

      // manifest.jsonをコピー
      copyFileSync(
        resolve(__dirname, 'manifest.json'),
        resolve(__dirname, 'dist/manifest.json'),
      )

      // popup.htmlをコピー
      copyFileSync(
        resolve(__dirname, 'src/popup/popup.html'),
        resolve(__dirname, 'dist/popup/popup.html'),
      )

      // styles.cssをコピー
      copyFileSync(
        resolve(__dirname, 'src/popup/styles.css'),
        resolve(__dirname, 'dist/popup/styles.css'),
      )

      // extraction-rules.jsonをコピー
      copyFileSync(
        resolve(__dirname, 'src/config/extraction-rules.json'),
        resolve(__dirname, 'dist/config/extraction-rules.json'),
      )
    },
  }
}

export default defineConfig({
  build: {
    emptyOutDir: true,
    outDir: 'dist',
    rollupOptions: {
      input: {
        'background/background': resolve(__dirname, 'src/background/background.ts'),
        'content-scripts/dmm-extractor': resolve(__dirname, 'src/content-scripts/dmm-extractor.ts'),
        'content-scripts/dlsite-extractor': resolve(__dirname, 'src/content-scripts/dlsite-extractor.ts'),
        'popup/popup': resolve(__dirname, 'src/popup/popup.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: (chunkInfo) => {
          // 共有チャンクを作らず、各エントリポイントに埋め込む
          return `chunks/[name]-[hash].js`;
        },
        assetFileNames: '[name].[ext]',
        manualChunks: (id) => {
          // すべてを個別のエントリポイントに含める
          return undefined;
        },
      },
      external: (id) => {
        // chromeのみを外部依存として扱い、他は全てバンドルする
        return id === 'chrome';
      },
    },
    target: 'esnext',
    minify: false,
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  plugins: [copyManifestPlugin()],
})
