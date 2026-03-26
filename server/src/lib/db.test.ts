import {
  remoteShareDedupeKeyForWork,
  remoteShareImageKeyForDedupeKey,
  toPublicWorkItem,
} from '@server/lib/db'

import { describe, expect, it } from 'vitest'

describe('toPublicWorkItem', () => {
  it('画像URLを期待形式で返す', () => {
    expect(
      toPublicWorkItem('device-id', {
        workId: 'work-1',
        title: 'Title',
      imageKey: 'device-id/work-1/thumbnail',
      thumbnailWidth: 400,
      thumbnailHeight: 225,
      erogamescapeId: 123,
      officialUrl: 'https://example.com/official',
      erogamescapeUrl: 'https://example.com/egs',
      seiyaUrl: null,
    }),
  ).toEqual({
    workId: 'work-1',
    title: 'Title',
    imageUrl: '/api/device/device-id/images/device-id%2Fwork-1%2Fthumbnail',
    width: 400,
    height: 225,
    officialUrl: 'https://example.com/official',
    erogamescapeUrl: 'https://example.com/egs',
  })
})
})

describe('remoteShare helpers', () => {
  it('erogamescape_id があれば egs キーを使う', () => {
    expect(
      remoteShareDedupeKeyForWork({
        workId: 'work-1',
        title: 'Title',
        erogamescapeId: 123,
      }),
    ).toBe('egs:123')
  })

  it('erogamescape_id がなければ workId にフォールバックする', () => {
    expect(
      remoteShareDedupeKeyForWork({
        workId: 'work-1',
        title: 'Title',
      }),
    ).toBe('work:work-1')
  })

  it('imageKey を dedupeKey から安定生成する', () => {
    expect(remoteShareImageKeyForDedupeKey('egs:123')).toBe('remote-share/egs%3A123/thumbnail')
  })
})
