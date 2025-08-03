import { copyFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// distディレクトリの作成を確実にする
mkdirSync(resolve(__dirname, '../dist/config'), { recursive: true })

// manifest.jsonをコピー
copyFileSync(
  resolve(__dirname, '../manifest.json'),
  resolve(__dirname, '../dist/manifest.json'),
)

console.log('Assets copied successfully')