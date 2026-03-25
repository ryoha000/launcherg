import { readFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const serverRoot = path.resolve(__dirname, '..')
const devVarsPath = path.join(serverRoot, '.dev.vars')

function parseDevVars(contents) {
  const result = new Map()

  for (const rawLine of contents.split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line || line.startsWith('#')) {
      continue
    }

    const equalsIndex = line.indexOf('=')
    if (equalsIndex <= 0) {
      continue
    }

    let key = line.slice(0, equalsIndex).trim()
    let value = line.slice(equalsIndex + 1).trim()

    if (key.startsWith('export ')) {
      key = key.slice('export '.length).trim()
    }

    if (
      (value.startsWith('"') && value.endsWith('"'))
      || (value.startsWith('\'') && value.endsWith('\''))
    ) {
      value = value.slice(1, -1)
    }

    result.set(key, value)
  }

  return result
}

function readDevVars() {
  const contents = readFileSync(devVarsPath, 'utf8')
  return parseDevVars(contents)
}

function getRequiredVar(vars, name) {
  const value = vars.get(name)?.trim()
  if (!value) {
    throw new Error(`.dev.vars に ${name} がありません`)
  }

  return value
}

function run() {
  const vars = readDevVars()
  const deployArgs = [
    'deploy',
    '--var',
    `R2_ACCOUNT_ID:${getRequiredVar(vars, 'R2_ACCOUNT_ID')}`,
    '--var',
    `R2_ACCESS_KEY_ID:${getRequiredVar(vars, 'R2_ACCESS_KEY_ID')}`,
    '--var',
    `R2_SECRET_ACCESS_KEY:${getRequiredVar(vars, 'R2_SECRET_ACCESS_KEY')}`,
  ]

  const wranglerEntrypoint = path.join(serverRoot, 'node_modules', 'wrangler', 'bin', 'wrangler.js')
  const result = spawnSync(process.execPath, [wranglerEntrypoint, ...deployArgs], {
    cwd: serverRoot,
    stdio: 'inherit',
    env: process.env,
    shell: false,
  })

  if (result.error) {
    throw result.error
  }

  process.exit(result.status ?? 1)
}

try {
  run()
}
catch (error) {
  const message = error instanceof Error ? error.message : String(error)
  console.error(message)
  process.exit(1)
}
