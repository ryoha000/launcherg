import { spawnSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import path from 'node:path'

const args = ['node_modules/@tauri-apps/cli/tauri.js', 'build']
if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
  if (process.env.CI && process.env.GITHUB_EVENT_NAME !== 'pull_request')
    throw new Error('TAURI_SIGNING_PRIVATE_KEY is required for a signed release build')
  mkdirSync('tmp', { recursive: true })
  const config = path.resolve('tmp/tauri-unsigned.json')
  writeFileSync(config, JSON.stringify({ bundle: { createUpdaterArtifacts: false } }))
  args.push('--config', config)
  console.log('Building an unsigned installer for local/PR verification; release signing runs in CI.')
}
const result = spawnSync(process.execPath, args, { stdio: 'inherit' })
if (result.error)
  throw result.error
process.exit(result.status ?? 1)
