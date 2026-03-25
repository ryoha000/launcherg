import { createSessionCookie, requireSession } from '@server/lib/cookies'

import { describe, expect, it } from 'vitest'

describe('session cookies', () => {
  it('署名済みクッキーを検証できる', async () => {
    const cookie = await createSessionCookie('secret', '00000000-0000-0000-0000-000000000000', 60)
    const request = new Request('https://example.com', {
      headers: {
        Cookie: cookie.split('; ')[0],
      },
    })

    await expect(
      requireSession(request, 'secret', '00000000-0000-0000-0000-000000000000'),
    ).resolves.toBeUndefined()
  })
})
