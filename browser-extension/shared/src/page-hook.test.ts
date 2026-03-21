import { describe, expect, it } from 'vitest'
import { createUrlBoundPayloadCache, isTypedWindowMessage } from './page-hook'

describe('page-hook', () => {
  it('想定した source/type の window message のみを受け入れる', () => {
    const matched = new MessageEvent('message', {
      data: {
        source: 'launcherg',
        type: 'matched',
        payload: { ok: true },
      },
    })
    const mismatchedType = new MessageEvent('message', {
      data: {
        source: 'launcherg',
        type: 'other',
      },
    })
    const mismatchedSource = new MessageEvent('message', {
      data: {
        source: 'launcherg',
        type: 'matched',
      },
    })
    Object.defineProperty(matched, 'source', { value: window })
    Object.defineProperty(mismatchedType, 'source', { value: window })
    Object.defineProperty(mismatchedSource, 'source', { value: null })

    expect(isTypedWindowMessage(matched, 'launcherg', 'matched')).toBe(true)
    expect(isTypedWindowMessage(mismatchedType, 'launcherg', 'matched')).toBe(false)
    expect(isTypedWindowMessage(mismatchedSource, 'launcherg', 'matched')).toBe(false)
  })

  it('uRL変更時に payload cache をリセットする', () => {
    const cache = createUrlBoundPayloadCache<{ value: string }>('https://example.com/one')

    cache.set({ value: 'first' })
    expect(cache.get()).toEqual({ value: 'first' })
    expect(cache.resetIfUrlChanged('https://example.com/one')).toBe(false)
    expect(cache.get()).toEqual({ value: 'first' })

    expect(cache.resetIfUrlChanged('https://example.com/two')).toBe(true)
    expect(cache.get()).toBeNull()
  })
})
