#!/usr/bin/env node

import { execSync } from 'child_process'
import { copyFileSync, existsSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const rootDir = join(__dirname, '..')
const tauriDir = join(rootDir, 'src-tauri')

const isDebug = process.argv.includes('--debug')
const profile = isDebug ? 'debug' : 'release'
const buildCommand = isDebug 
  ? 'cargo build --bin native-messaging-host'
  : 'cargo build --release --bin native-messaging-host'

console.log(`Building native-messaging-host (${profile} mode)...`)

try {
  // Change to src-tauri directory and build
  execSync(buildCommand, {
    cwd: tauriDir,
    stdio: 'inherit'
  })

  // Copy the executable to src-tauri root
  const exeName = process.platform === 'win32' ? 'native-messaging-host.exe' : 'native-messaging-host'
  const sourcePath = join(tauriDir, 'target', profile, exeName)
  const destPath = join(tauriDir, exeName)

  if (!existsSync(sourcePath)) {
    throw new Error(`Build artifact not found: ${sourcePath}`)
  }

  console.log(`Copying ${sourcePath} to ${destPath}`)
  copyFileSync(sourcePath, destPath)

  console.log('✅ Native messaging host built and copied successfully!')
} catch (error) {
  console.error('❌ Build failed:', error.message)
  process.exit(1)
}