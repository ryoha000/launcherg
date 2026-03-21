import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('dmm marker attributes', () => {
  it('content-script と network-hook が kebab-case の data 属性名を使う', () => {
    const mainSource = readFileSync(resolve(__dirname, '../src/main.ts'), 'utf-8')
    const networkHookSource = readFileSync(resolve(__dirname, '../src/network-hook.ts'), 'utf-8')

    expect(mainSource).toContain('data-launcherg-dmm-content-script-installed')
    expect(networkHookSource).toContain('data-launcherg-dmm-network-hook-installed')
  })
})
