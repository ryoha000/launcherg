import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const extension = path.join(root, 'browser-extension/dist')
const manifest = JSON.parse(readFileSync(path.join(extension, 'manifest.json'), 'utf8'))
const config = JSON.parse(readFileSync(path.join(root, 'src-tauri/tauri.conf.json'), 'utf8'))
if (manifest.version !== config.version)
  throw new Error('Extension and desktop versions must match')
const required = [manifest.background.service_worker, manifest.action.default_popup,
  ...manifest.content_scripts.flatMap(script => script.js),
  ...manifest.web_accessible_resources.flatMap(entry => entry.resources)]
for (const file of required) {
  if (!existsSync(path.join(extension, file)))
    throw new Error('Missing extension asset: ' + file)
}
const host = path.join(root, 'src-tauri/native-messaging-host.exe')
if (!existsSync(host))
  throw new Error('Build the native host with task build:extension first')
const target = path.join(root, 'src-tauri/resources/integrations')
if (path.dirname(target) !== path.join(root, 'src-tauri/resources'))
  throw new Error('Invalid staging directory')
rmSync(target, { recursive: true, force: true })
mkdirSync(target, { recursive: true })
cpSync(extension, path.join(target, 'browser-extension'), { recursive: true, filter: source => !source.endsWith('.map') })
cpSync(host, path.join(target, 'native-messaging-host.exe'))
console.log('Staged integration resources for ' + config.version + ': ' + readdirSync(target).join(', '))
