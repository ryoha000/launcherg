import { internalServerError } from '@server/lib/errors'

const SIGV4_ALGORITHM = 'AWS4-HMAC-SHA256'
const SIGV4_REGION = 'auto'
const SIGV4_SERVICE = 's3'
const DEFAULT_PRESIGN_TTL_SECONDS = 15 * 60

function encodeRfc3986(value: string): string {
  return encodeURIComponent(value).replace(/[!'()*]/g, char => `%${char.charCodeAt(0).toString(16).toUpperCase()}`)
}

export function encodeObjectKey(key: string): string {
  return key
    .split('/')
    .map(segment => encodeRfc3986(segment))
    .join('/')
}

function formatAmzDate(date: Date): { amzDate: string, dateStamp: string } {
  const iso = date.toISOString()
  return {
    amzDate: iso.replace(/[-:]/g, '').replace(/\.\d{3}Z$/, 'Z'),
    dateStamp: iso.slice(0, 10).replace(/-/g, ''),
  }
}

async function sha256Hex(input: string): Promise<string> {
  const bytes = new TextEncoder().encode(input)
  const hash = await crypto.subtle.digest('SHA-256', bytes)
  return Array.from(new Uint8Array(hash), byte => byte.toString(16).padStart(2, '0')).join('')
}

async function hmacSha256(key: ArrayBuffer, data: string): Promise<ArrayBuffer> {
  const importedKey = await crypto.subtle.importKey(
    'raw',
    key,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  return await crypto.subtle.sign('HMAC', importedKey, new TextEncoder().encode(data))
}

async function deriveSigningKey(secretAccessKey: string, dateStamp: string): Promise<ArrayBuffer> {
  const kDate = await hmacSha256(new TextEncoder().encode(`AWS4${secretAccessKey}`).buffer, dateStamp)
  const kRegion = await hmacSha256(kDate, SIGV4_REGION)
  const kService = await hmacSha256(kRegion, SIGV4_SERVICE)
  return await hmacSha256(kService, 'aws4_request')
}

function requireEnvVar(name: keyof Env, value: string | undefined): string {
  const trimmed = value?.trim()
  if (!trimmed) {
    internalServerError(`Missing required R2 environment variable: ${name}`)
  }

  return trimmed
}

function buildCanonicalQueryString(params: Record<string, string>): string {
  return Object.entries(params)
    .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
    .map(([key, value]) => `${encodeRfc3986(key)}=${encodeRfc3986(value)}`)
    .join('&')
}

export async function createR2PresignedPutUrl(
  env: Env,
  objectKey: string,
  contentType: string,
): Promise<string> {
  const parsedExpiresSeconds = Number(env.R2_PRESIGN_TTL_SECONDS ?? DEFAULT_PRESIGN_TTL_SECONDS)
  const expiresSeconds = Number.isFinite(parsedExpiresSeconds) && parsedExpiresSeconds > 0
    ? parsedExpiresSeconds
    : DEFAULT_PRESIGN_TTL_SECONDS
  const { amzDate, dateStamp } = formatAmzDate(new Date())
  const bucketName = requireEnvVar('R2_BUCKET_NAME', env.R2_BUCKET_NAME)
  const accountId = requireEnvVar('R2_ACCOUNT_ID', env.R2_ACCOUNT_ID)
  const accessKeyId = requireEnvVar('R2_ACCESS_KEY_ID', env.R2_ACCESS_KEY_ID)
  const secretAccessKey = requireEnvVar('R2_SECRET_ACCESS_KEY', env.R2_SECRET_ACCESS_KEY)
  let host: string
  let canonicalUri: string
  let baseUrl: string

  const customEndpoint = env.R2_CUSTOM_ENDPOINT
  if (customEndpoint) {
    const url = new URL(customEndpoint)
    host = url.host
    canonicalUri = `/${bucketName}/${encodeObjectKey(objectKey)}`
    baseUrl = `${customEndpoint.replace(/\/$/, '')}${canonicalUri}`
  } else {
    host = `${bucketName}.${accountId}.r2.cloudflarestorage.com`
    canonicalUri = `/${encodeObjectKey(objectKey)}`
    baseUrl = `https://${host}${canonicalUri}`
  }
  const signedHeaders = 'content-type;host'
  const credentialScope = `${dateStamp}/${SIGV4_REGION}/${SIGV4_SERVICE}/aws4_request`
  const queryParams = {
    'X-Amz-Algorithm': SIGV4_ALGORITHM,
    'X-Amz-Credential': `${accessKeyId}/${credentialScope}`,
    'X-Amz-Date': amzDate,
    'X-Amz-Expires': String(expiresSeconds),
    'X-Amz-SignedHeaders': signedHeaders,
    'X-Amz-Content-Sha256': 'UNSIGNED-PAYLOAD',
  }
  const canonicalQueryString = buildCanonicalQueryString(queryParams)
  const canonicalHeaders = `content-type:${contentType.trim()}\nhost:${host}\n`
  const canonicalRequest = [
    'PUT',
    canonicalUri,
    canonicalQueryString,
    canonicalHeaders,
    signedHeaders,
    'UNSIGNED-PAYLOAD',
  ].join('\n')
  const stringToSign = [
    SIGV4_ALGORITHM,
    amzDate,
    credentialScope,
    await sha256Hex(canonicalRequest),
  ].join('\n')
  const signingKey = await deriveSigningKey(secretAccessKey, dateStamp)
  const signatureBytes = await hmacSha256(signingKey, stringToSign)
  const signature = Array.from(new Uint8Array(signatureBytes), byte => byte.toString(16).padStart(2, '0')).join('')

  return `${baseUrl}?${canonicalQueryString}&X-Amz-Signature=${signature}`
}
import type { Env } from '@server/env'
