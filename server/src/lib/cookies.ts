import { hmacSha256Hex } from '@server/lib/crypto'
import { unauthorized } from '@server/lib/errors'

const SESSION_COOKIE_NAME = 'launcherg_remote_session'

interface SessionPayload {
  deviceId: string
  expiresAt: number
}

export async function createSessionCookie(
  sessionSecret: string,
  deviceId: string,
  ttlSeconds: number,
): Promise<string> {
  const payload: SessionPayload = {
    deviceId,
    expiresAt: Date.now() + (ttlSeconds * 1000),
  }
  const body = btoa(JSON.stringify(payload))
  const signature = await hmacSha256Hex(sessionSecret, body)
  const token = `${body}.${signature}`
  return [
    `${SESSION_COOKIE_NAME}=${token}`,
    'HttpOnly',
    'Path=/',
    'SameSite=Lax',
    `Max-Age=${ttlSeconds}`,
  ].join('; ')
}

export async function requireSession(
  request: Request,
  sessionSecret: string,
  deviceId: string,
): Promise<void> {
  const cookies = parseCookieHeader(request.headers.get('Cookie'))
  const token = cookies[SESSION_COOKIE_NAME]

  if (!token) {
    unauthorized()
  }

  const [body, signature] = token.split('.', 2)
  if (!body || !signature) {
    unauthorized()
  }

  const expected = await hmacSha256Hex(sessionSecret, body)
  if (expected !== signature) {
    unauthorized()
  }

  const payload = JSON.parse(atob(body)) as SessionPayload
  if (payload.deviceId !== deviceId || payload.expiresAt <= Date.now()) {
    unauthorized()
  }
}

function parseCookieHeader(header: string | null): Record<string, string> {
  if (!header) {
    return {}
  }

  return header
    .split(';')
    .map(segment => segment.trim())
    .filter(Boolean)
    .reduce<Record<string, string>>((acc, segment) => {
      const [key, ...rest] = segment.split('=')
      acc[key] = rest.join('=')
      return acc
    }, {})
}

export { SESSION_COOKIE_NAME }
