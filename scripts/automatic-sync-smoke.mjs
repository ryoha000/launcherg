// Local synthetic site only: no account, store navigation, manual sync, or mocked native messaging.
import { chromium } from '../browser-extension/node_modules/playwright/index.mjs'
import { execFileSync, spawnSync } from 'node:child_process'
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { tmpdir } from 'node:os'
import path from 'node:path'
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'

const root = mkdtempSync(path.join(tmpdir(), 'launcherg-auto-sync-'))
const extension = path.join(root, 'extension')
cpSync('browser-extension/dist', extension, { recursive: true })
const host = path.resolve('src-tauri/target/release/native-messaging-host.exe')
const hostName = 'moe.ryoha.launcherg.auto_test_' + process.pid
const registry = 'HKCU\\SOFTWARE\\Google\\Chrome\\NativeMessagingHosts\\' + hostName
const dataDir = path.join(root, 'data')
const downloadsDir = path.join(root, 'downloads')
mkdirSync(downloadsDir)
const fixtureZip = path.join(root, 'fixture.zip')
execFileSync('python', ['-c', 'import sys,zipfile; z=zipfile.ZipFile(sys.argv[1],"w"); z.writestr("game/game.exe",b"MZ fixture; never executed"); z.close()', fixtureZip], { windowsHide: true })
let requests = 0
let zipRequests = 0
const server = createServer((req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*')
  if (req.url === '/api/v3/download?workno=RJ-CACHE-TEST') {
    zipRequests++
    res.setHeader('Content-Type', 'application/zip')
    res.setHeader('Content-Disposition', 'attachment; filename="game.zip"')
    res.end(readFileSync(fixtureZip))
  }
  else if (/^\/api\/v[34]\/content\/works/.test(req.url)) {
    requests++
    if (req.url.startsWith('/api/v4') && req.method !== 'POST') {
      res.writeHead(405)
      res.end()
      return
    }
    res.setHeader('Content-Type', 'application/json')
    const imageField = req.url.startsWith('/api/v4') ? 'images' : 'work_files'
    res.end(JSON.stringify({ works: [{ workno: 'RJ-AUTO-TEST', site_id: 'maniax', file_type: 'EXE', name: { ja_JP: 'Automatic sync test' }, [imageField]: { main: origin + '/image.png' } }] }))
  }
  else if (req.url === '/ajax/v1/library') {
    res.setHeader('Content-Type', 'application/json')
    res.end(JSON.stringify({ error: null, body: { totalCount: 1, library: [{ productId: 'DMM-AUTO-TEST', libraryProductType: 'single', floor: 'pcgame', title: 'DMM automatic fixture', packageImageUrl: origin + '/image.png' }] } }))
  }
  else if (req.url === '/image.png') {
    res.setHeader('Content-Type', 'image/png')
    res.end(Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aJRkAAAAASUVORK5CYII=', 'base64'))
  }
  else if (req.url === '/egs') {
    res.setHeader('Content-Type', 'text/html; charset=utf-8')
    res.end('<table><tr><th>no results</th></tr></table>')
  }
  else {
    res.setHeader('Content-Type', 'text/html; charset=utf-8')
    res.setHeader('Content-Security-Policy', "default-src 'none'; script-src 'nonce-test'; connect-src 'self'")
    // Starts during initial HTML parsing, before DOMContentLoaded.
    let request = ''
    if (req.url.startsWith('/work/RJ-CACHE-TEST'))
      request = 'history.replaceState({}, "", location.href.replace("/tree", "")); setTimeout(() => { const link = document.createElement("a"); link.href = "/api/v3/download?workno=RJ-CACHE-TEST"; link.target = "_blank"; link.textContent = "ZIPダウンロード"; document.body.append(link) }, 1500);'
    else if (req.url === '/dmm/library')
      request = 'fetch("/ajax/v1/library").then(r=>r.json());'
    else if (req.url === '/dmm/xhr-library')
      request = 'const xhr = new XMLHttpRequest(); xhr.open("GET", "/ajax/v1/library"); xhr.responseType = "json"; xhr.send();'
    else if (req.url === '/')
      request = 'const opened = indexedDB.open("Play_TEST", 1); opened.onsuccess = () => { window.fixtureDatabase = opened.result };'
    else if (req.url.startsWith('/legacy'))
      request = 'const xhr = new XMLHttpRequest(); xhr.open("GET", "/api/v3/content/works"); xhr.responseType = "json"; xhr.send();'
    else if (req.url === '/library')
      request = 'fetch("/api/v4/content/works", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ worknos: ["RJ-AUTO-TEST"] }) }).then(r=>r.json());'
    res.end('<!doctype html><script nonce="test">' + request + '</script><body>Local library fixture</body>')
  }
})
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
const origin = 'http://127.0.0.1:' + server.address().port
const manifest = JSON.parse(readFileSync(path.join(extension, 'manifest.json')))
manifest.content_scripts = manifest.content_scripts.map(item => item.js.some(file => file.includes('dmm'))
  ? { ...item, matches: [origin + '/dmm/*'] }
  : { ...item, matches: [origin + '/*'], exclude_matches: [origin + '/dmm/*'] })
manifest.host_permissions = [origin + '/*']
manifest.web_accessible_resources = []
writeFileSync(path.join(extension, 'manifest.json'), JSON.stringify(manifest))
const backgroundPath = path.join(extension, 'background/background.js')
writeFileSync(backgroundPath, readFileSync(backgroundPath, 'utf8')
  .replaceAll('moe.ryoha.launcherg.extension_host', hostName)
  .replaceAll('https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php', origin + '/egs'))
const extensionId = [...createHash('sha256').update(Buffer.from(manifest.key, 'base64')).digest().subarray(0, 16)]
  .flatMap(byte => [String.fromCharCode(97 + (byte >> 4)), String.fromCharCode(97 + (byte & 15))]).join('')
const nativeManifest = path.join(root, 'native.json')
writeFileSync(nativeManifest, JSON.stringify({ name: hostName, description: 'Local automatic sync test', path: host, type: 'stdio', allowed_origins: ['chrome-extension://' + extensionId + '/'] }))
assert.notEqual(spawnSync('reg.exe', ['query', registry], { windowsHide: true }).status, 0)
let context
try {
  execFileSync('reg.exe', ['add', registry, '/ve', '/t', 'REG_SZ', '/d', nativeManifest, '/f'], { windowsHide: true, stdio: 'pipe' })
  context = await chromium.launchPersistentContext(path.join(root, 'browser'), {
    channel: 'chromium', headless: true, downloadsPath: downloadsDir,
    args: ['--disable-extensions-except=' + extension, '--load-extension=' + extension],
    env: { ...process.env, LAUNCHERG_DATA_DIR: dataDir },
  })
  const page = await context.newPage()
  const errors = []
  page.on('pageerror', error => errors.push(error.message))
  const logPath = path.join(dataDir, 'diagnostics/browser-sync.jsonl')
  const events = () => existsSync(logPath) ? readFileSync(logPath, 'utf8').trim().split('\n').filter(Boolean).map(line => JSON.parse(line)) : []
  const waitForAutomaticSync = async (count, store = 'dlsite') => {
    const deadline = Date.now() + 30_000
    while (Date.now() < deadline) {
      if (events().filter(event => event.store === store && event.stage === 'sync_finished' && event.trigger === 'auto').length >= count)
        return
      await new Promise(resolve => setTimeout(resolve, 250))
    }
    throw new Error('Automatic sync timed out: ' + JSON.stringify({ errors, events: events() }))
  }
  await page.goto(origin + '/seed')
  await page.evaluate(async (origin) => {
    for (const [name, workno] of [['Play_INACTIVE', 'RJ-INACTIVE'], ['Play_TEST', 'RJ-CACHE-TEST']]) {
      await new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1)
        request.onupgradeneeded = () => request.result.createObjectStore('Purchase', { keyPath: 'workno' })
        request.onerror = () => reject(request.error)
        request.onsuccess = () => {
          const db = request.result
          const tx = db.transaction('Purchase', 'readwrite')
          tx.objectStore('Purchase').put({ workno, site_id: 'maniax', file_type: 'EXE', name: { ja_JP: 'Cached library fixture' }, images: { main: origin + '/image.png' } })
          tx.oncomplete = () => { db.close(); resolve() }
        }
      })
    }
  }, origin)
  await page.goto(origin + '/library')
  await waitForAutomaticSync(1)
  await page.reload()
  await waitForAutomaticSync(2)
  await page.goto(origin + '/legacy-library')
  await waitForAutomaticSync(3)
  // A returning user sees a cached library and the site's own code sends no API request.
  await page.goto(origin + '/')
  await waitForAutomaticSync(4)
  // Simulate a committed cache refresh by the site, without calling manual sync.
  await page.evaluate(origin => new Promise((resolve) => {
    const tx = window.fixtureDatabase.transaction('Purchase', 'readwrite')
    tx.objectStore('Purchase').put({ workno: 'RJ-CACHE-ADDED', site_id: 'maniax', file_type: 'EXE', name: { ja_JP: 'New purchase fixture' }, images: { main: origin + '/image.png' } })
    tx.oncomplete = () => resolve()
  }), origin)
  await waitForAutomaticSync(5)
  await page.reload()
  await waitForAutomaticSync(6)
  assert.equal(requests, 3, 'Cached library must not cause an extra API request')
  const versions = await page.evaluate(async () => (await indexedDB.databases()).map(db => ({ name: db.name, version: db.version })).sort((a, b) => a.name.localeCompare(b.name)))
  assert.deepEqual(versions, [{ name: 'Play_INACTIVE', version: 1 }, { name: 'Play_TEST', version: 1 }])
  await page.goto(origin + '/dmm/library')
  await waitForAutomaticSync(1, 'dmm')
  await page.reload()
  await waitForAutomaticSync(2, 'dmm')
  await page.goto(origin + '/dmm/xhr-library')
  await waitForAutomaticSync(3, 'dmm')
  assert.ok(!events().some(event => event.trigger === 'manual'), 'Must not use manual sync')
  assert.ok(!events().some(event => ['sync_failed', 'hook_missing'].includes(event.stage)))
  assert.deepEqual(errors, [], 'Page code must remain functional under strict CSP')
  const ids = JSON.parse(execFileSync('python', ['-c', 'import sqlite3,sys,json; c=sqlite3.connect(sys.argv[1]); print(json.dumps([r[0] for r in c.execute("SELECT store_id FROM dlsite_works ORDER BY store_id")])); c.close()', path.join(dataDir, 'launcherg_sqlite.db3')], { encoding: 'utf8', windowsHide: true }).trim())
  assert.deepEqual(ids, ['RJ-AUTO-TEST', 'RJ-CACHE-ADDED', 'RJ-CACHE-TEST'], 'Reload is idempotent and inactive account caches are not imported')
  const dmmIds = JSON.parse(execFileSync('python', ['-c', 'import sqlite3,sys,json; c=sqlite3.connect(sys.argv[1]); print(json.dumps([r[0] for r in c.execute("SELECT store_id FROM dmm_works ORDER BY store_id")])); c.close()', path.join(dataDir, 'launcherg_sqlite.db3')], { encoding: 'utf8', windowsHide: true }).trim())
  assert.deepEqual(dmmIds, ['DMM-AUTO-TEST'])
  // Match the site's current URL normalization and delayed download link. The
  // browser downloads real fixture bytes and the host performs real extraction.
  const downloadSession = await context.newCDPSession(page)
  await downloadSession.send('Browser.setDownloadBehavior', { behavior: 'allow', downloadPath: downloadsDir })
  const payload = { type: 'download', value: { game: { storeId: 'RJ-CACHE-TEST' } } }
  await page.goto(origin + '/work/RJ-CACHE-TEST/tree?launcherg=' + encodeURIComponent(JSON.stringify(payload)))
  const receiptDir = path.join(dataDir, 'download-jobs')
  const deadline = Date.now() + 30_000
  let receipt
  while (Date.now() < deadline) {
    if (existsSync(receiptDir)) {
      const filename = readdirSync(receiptDir).find(name => name.endsWith('.json'))
      if (filename) {
        receipt = JSON.parse(readFileSync(path.join(receiptDir, filename), 'utf8'))
        if (receipt.completed_path)
          break
      }
    }
    await new Promise(resolve => setTimeout(resolve, 250))
  }
  assert.ok(receipt?.completed_path, 'Real download/install did not complete: ' + JSON.stringify({ receipt, events: events().slice(-10) }))
  assert.equal(zipRequests, 1, 'URL normalization must not start duplicate downloads')
  assert.ok(existsSync(path.join(downloadsDir, 'game.zip')), 'Original ZIP must be retained')
  assert.ok(existsSync(path.join(receipt.completed_path, 'game', 'game.exe')), 'Fixture executable must be extracted')
  assert.ok(page.isClosed(), 'The content tab closes after the browser accepts the download')
  assert.deepEqual(errors, [])
  console.log('PASS: DLsite v4 POST / v3 XHR / active account Purchase cache / committed cache refresh and DMM fetch / XHR + strict CSP → automatic sync → real Chrome Native Messaging → isolated DB; no extra API requests, inactive account imports, schema changes, or duplicate works')
  console.log('PASS: DLsite current detail URL + delayed ZIP link → real Chrome download → real Native Host extraction/registration; original ZIP retained')
  console.log('Local evidence: ' + logPath)
}
finally {
  await context?.close()
  server.close()
  execFileSync('reg.exe', ['delete', registry, '/f'], { windowsHide: true, stdio: 'pipe' })
}
