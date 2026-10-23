import { cpSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import path from 'node:path'

const version = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8')).version
const source = 'src-tauri/target/release/bundle/msi'
const files = readdirSync(source).filter(file => file.includes('_' + version + '_') && /\.msi(\.zip)?(\.sig)?$/.test(file))
if (!files.some(file => file.endsWith('.msi')))
  throw new Error('No MSI installer found')
mkdirSync('release-artifacts', { recursive: true })
for (const file of files)
  cpSync(path.join(source, file), path.join('release-artifacts', file))
const updater = files.find(file => file.endsWith('.msi.zip'))
if (process.argv.includes('--require-signature') && (!updater || !files.includes(updater + '.sig')))
  throw new Error('Signed updater archive is missing')
if (updater && files.includes(updater + '.sig')) {
  const signature = readFileSync(path.join(source, updater + '.sig'), 'utf8').trim()
  const url = 'https://github.com/ryoha000/launcherg/releases/download/' + version + '/' + updater
  writeFileSync('release-artifacts/updater.json', JSON.stringify({
    version, notes: 'Launcherg ' + version + ' prerelease', pub_date: new Date().toISOString(),
    url, signature, platforms: { 'windows-x86_64': { url, signature } },
  }, null, 2) + '\n')
}
console.log('Collected: ' + files.join(', '))
