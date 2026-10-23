import { readFileSync } from 'node:fs'
const json = path => JSON.parse(readFileSync(path, 'utf8'))
const version = json('src-tauri/tauri.conf.json').version
if (!/^\d+\.\d+\.\d+$/.test(version))
  throw new Error('Release version must be x.y.z')
for (const file of ['package.json', 'browser-extension/manifest.json']) {
  if (json(file).version !== version)
    throw new Error(file + ' has a different version')
}
for (const file of ['src-tauri/Cargo.toml', 'src-tauri/native-messaging-host/Cargo.toml']) {
  if (readFileSync(file, 'utf8').match(/^version = "([^"]+)"/m)?.[1] !== version)
    throw new Error(file + ' has a different version')
}
if (process.env.GITHUB_REF_TYPE === 'tag' && process.env.GITHUB_REF_NAME !== version)
  throw new Error('Tag must match the version committed in the repository')
console.log('Release version: ' + version)
