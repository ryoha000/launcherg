import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('dlsite marker attributes', () => {
  it('content-script と network-hook が kebab-case の data 属性名を使う', () => {
    const mainSource = readFileSync(resolve(__dirname, 'main.ts'), 'utf-8')
    const networkHookSource = readFileSync(resolve(__dirname, 'network-hook.ts'), 'utf-8')

    expect(mainSource).toContain('data-launcherg-dlsite-content-script-installed')
    expect(networkHookSource).toContain('data-launcherg-dlsite-network-hook-installed')
  })
})
