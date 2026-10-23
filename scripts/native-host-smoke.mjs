import { spawnSync } from 'node:child_process'
const result = spawnSync('python', ['scripts/native-host-smoke.py'], { stdio: 'inherit' })
if (result.error)
  throw result.error
process.exit(result.status ?? 1)
