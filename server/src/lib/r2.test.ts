import type { Env } from '@server/env'
import { createR2PresignedPutUrl } from '@server/lib/r2'

import { describe, expect, it } from 'vitest'

function createEnv(overrides: Partial<Env> = {}): Env {
  return {
    DB: {} as D1Database,
    IMAGES: {} as R2Bucket,
    ASSETS: {} as Fetcher,
    SESSION_SECRET: 'session-secret',
    R2_ACCOUNT_ID: 'account-id',
    R2_ACCESS_KEY_ID: 'access-key-id',
    R2_SECRET_ACCESS_KEY: 'secret-access-key',
    R2_BUCKET_NAME: 'launcherg-images',
    R2_PRESIGN_TTL_SECONDS: '900',
    ...overrides,
  }
}

describe('createR2PresignedPutUrl', () => {
  it('R2_ACCOUNT_ID が未設定なら明示的に失敗する', async () => {
    await expect(
      createR2PresignedPutUrl(createEnv({ R2_ACCOUNT_ID: '' }), 'remote-share/egs%3A7175/thumbnail', 'image/png'),
    ).rejects.toThrow('Missing required R2 environment variable: R2_ACCOUNT_ID')
  })

  it('設定済みなら accountId を含む URL を生成する', async () => {
    const url = await createR2PresignedPutUrl(
      createEnv(),
      'remote-share/egs%3A7175/thumbnail',
      'image/png',
    )

    expect(url).toContain('https://launcherg-images.account-id.r2.cloudflarestorage.com/')
    expect(url).toContain('X-Amz-Credential=access-key-id%2F')
  })
})
