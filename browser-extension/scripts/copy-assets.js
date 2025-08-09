import { copyFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// distディレクトリの作成を確実にする
mkdirSync(resolve(__dirname, '../dist/config'), { recursive: true })
mkdirSync(resolve(__dirname, '../dist/icons'), { recursive: true })

// manifest.jsonをコピー
copyFileSync(
  resolve(__dirname, '../manifest.json'),
  resolve(__dirname, '../dist/manifest.json'),
)

console.log('Assets copied successfully')

// アイコンをコピー（拡張用）
try {
  const sourceIcon = resolve(__dirname, '../../public/icon.png')
  const destIcon32 = resolve(__dirname, '../dist/icons/icon32.png')
  const destIcon32Error = resolve(__dirname, '../dist/icons/icon32_error.png')

  copyFileSync(sourceIcon, destIcon32)
  copyFileSync(sourceIcon, destIcon32Error)
  console.log('Icons copied successfully')
}
catch (e) {
  console.warn('Icon copy skipped or failed:', e?.message || e)
}