import { describe, expect, it } from 'vitest'
import { buildWorkOpenDeepLink, parseDeepLinkUrl } from './deepLink'

describe('deepLink', () => {
  it('work の deep link URL を生成できる', () => {
    expect(buildWorkOpenDeepLink('123')).toBe('launcherg://works/open?id=123')
    expect(buildWorkOpenDeepLink('123', { play: true })).toBe('launcherg://works/open?id=123&play=true')
    expect(buildWorkOpenDeepLink('123', { play: true, gamename: 'ゲームA' })).toBe(
      'launcherg://works/open?id=123&gamename=%E3%82%B2%E3%83%BC%E3%83%A0A&play=true',
    )
  })

  it('work の deep link を work detail へ変換できる', () => {
    expect(parseDeepLinkUrl('launcherg://works/open?id=123')).toEqual({
      kind: 'works',
      key: '123',
      path: '/works/123',
      play: false,
    })
    expect(parseDeepLinkUrl('launcherg://works/open?id=123&play=true')).toEqual({
      kind: 'works',
      key: '123',
      path: '/works/123?play=true',
      play: true,
    })
    expect(parseDeepLinkUrl('launcherg://works/open?id=123&gamename=%E3%82%B2%E3%83%BC%E3%83%A0A&play=true')).toEqual({
      kind: 'works',
      key: '123',
      path: '/works/123?gamename=%E3%82%B2%E3%83%BC%E3%83%A0A&play=true',
      play: true,
    })
  })

  it('settings の deep link を singleton route へ変換できる', () => {
    expect(parseDeepLinkUrl('launcherg://settings')).toEqual({
      kind: 'settings',
      path: '/settings',
    })
  })

  it('不正な deep link は無視する', () => {
    expect(parseDeepLinkUrl('https://example.com/works/open?id=123')).toBeNull()
    expect(parseDeepLinkUrl('launcherg://works/open')).toBeNull()
    expect(parseDeepLinkUrl('launcherg://unknown')).toBeNull()
  })
})
