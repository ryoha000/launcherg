import { describe, expect, it } from 'vitest'
import { extractCidFromUrl } from '../src/utils'

describe('dmm utils', () => {
  it('extractCidFromUrl', () => {
    expect(extractCidFromUrl('https://x?cid=aa_bb&x=1')).toBe('aa_bb')
    expect(extractCidFromUrl('?x=1&cid=zz')).toBe('zz')
    expect(extractCidFromUrl('')).toBeNull()
  })
})
