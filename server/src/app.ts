import type { Env } from '@server/env'

import { createSessionCookie, requireSession } from '@server/lib/cookies'
import { sha256Hex } from '@server/lib/crypto'
import {
  clearDeviceSnapshots,
  findDeviceById,
  findDeviceBySecretHash,
  insertDevice,
  insertRemoteShareImage,
  listDeviceSnapshots,
  listRemoteShareImagesByDedupeKeys,
  remoteShareDedupeKeyForWork,
  remoteShareImageKeyForDedupeKey,
  toPublicWorkItem,
  upsertDeviceSnapshots,
} from '@server/lib/db'
import { badRequest, notFound, unauthorized } from '@server/lib/errors'
import { createR2PresignedPutUrl, encodeObjectKey } from '@server/lib/r2'
import {
  deviceRegisterInputSchema,
  deviceSessionInputSchema,
  deviceWorksSyncCommitInputSchema,
  deviceWorksSyncPrepareInputSchema,
  type RemoteShareUploadTarget,
} from '@server/shared/schema'

const SESSION_DEFAULT_TTL_SECONDS = 60 * 60

export async function handleRequest(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url)

  if (url.pathname.startsWith('/api/device/') && url.pathname.includes('/images/')) {
    return handleImageRequest(request, env, url)
  }

  if (url.pathname === '/api/device/register' && request.method === 'POST') {
    return handleRegisterDevice(request, env)
  }

  if (url.pathname === '/api/device/session' && request.method === 'POST') {
    return handleCreateSession(request, env)
  }

  const worksListMatch = url.pathname.match(/^\/api\/device\/([^/]+)\/works$/)
  if (worksListMatch && request.method === 'GET') {
    return handleListWorks(request, env, worksListMatch[1])
  }

  const worksLaunchMatch = url.pathname.match(/^\/api\/device\/([^/]+)\/works\/([^/]+)\/launch$/)
  if (worksLaunchMatch && request.method === 'POST') {
    return handleLaunchWork(request, env, worksLaunchMatch[1], worksLaunchMatch[2])
  }

  const launchBrokerMatch = url.pathname.match(/^\/api\/device\/([^/]+)\/launch-broker$/)
  if (launchBrokerMatch && request.method === 'GET') {
    return handleLaunchBrokerConnect(request, env, launchBrokerMatch[1], url)
  }

  const worksSyncPrepareMatch = url.pathname.match(/^\/api\/device\/([^/]+)\/works\/sync\/prepare$/)
  if (worksSyncPrepareMatch && request.method === 'POST') {
    return handlePrepareSyncWorks(request, env, worksSyncPrepareMatch[1])
  }

  const worksSyncCommitMatch = url.pathname.match(/^\/api\/device\/([^/]+)\/works\/sync\/commit$/)
  if (worksSyncCommitMatch && request.method === 'POST') {
    return handleCommitSyncWorks(request, env, worksSyncCommitMatch[1])
  }

  return env.ASSETS.fetch(request)
}

async function handleRegisterDevice(request: Request, env: Env): Promise<Response> {
  const input = deviceRegisterInputSchema.parse(await request.json())
  const secretHash = await sha256Hex(input.deviceSecret)
  const existing = await findDeviceBySecretHash(env.DB, secretHash)

  if (existing) {
    return Response.json({ deviceId: existing.deviceId })
  }

  const deviceId = crypto.randomUUID()
  await insertDevice(env.DB, deviceId, secretHash)
  return Response.json({ deviceId })
}

async function handleCreateSession(request: Request, env: Env): Promise<Response> {
  const input = deviceSessionInputSchema.parse(await request.json())
  const secretHash = await sha256Hex(input.deviceSecret)
  const device = await findDeviceById(env.DB, input.deviceId)

  if (!device || device.secretHash !== secretHash) {
    unauthorized('deviceSecret is invalid')
  }

  const ttlSeconds = Number(env.SESSION_TTL_SECONDS ?? SESSION_DEFAULT_TTL_SECONDS)
  const cookie = await createSessionCookie(env.SESSION_SECRET, input.deviceId, ttlSeconds)

  return new Response(null, {
    status: 204,
    headers: {
      'Set-Cookie': cookie,
    },
  })
}

async function handleListWorks(
  request: Request,
  env: Env,
  deviceId: string,
): Promise<Response> {
  const device = await findDeviceById(env.DB, deviceId)
  if (!device) {
    notFound('device not found')
  }

  await requireSession(request, env.SESSION_SECRET, deviceId)
  const rows = await listDeviceSnapshots(env.DB, deviceId)

  return Response.json({
    deviceId,
    lastSyncedAt: device.lastSyncedAt,
    works: rows.map(row => toPublicWorkItem(deviceId, row)),
  })
}

async function handlePrepareSyncWorks(
  request: Request,
  env: Env,
  deviceId: string,
): Promise<Response> {
  const body = deviceWorksSyncPrepareInputSchema.parse(await request.json())
  const secretHash = await sha256Hex(body.deviceSecret)
  const device = await findDeviceById(env.DB, deviceId)

  if (!device) {
    notFound('device not found')
  }

  if (device.secretHash !== secretHash) {
    unauthorized('deviceSecret is invalid')
  }

  const dedupeKeys = body.works
    .filter(work => Boolean(work.thumbnail))
    .map(work => remoteShareDedupeKeyForWork(work))
  const existingImages = await listRemoteShareImagesByDedupeKeys(env.DB, dedupeKeys)
  const seen = new Set<string>()
  const uploadTargets: RemoteShareUploadTarget[] = []

  for (const work of body.works) {
    if (!work.thumbnail) {
      continue
    }

    const dedupeKey = remoteShareDedupeKeyForWork(work)
    if (seen.has(dedupeKey)) {
      continue
    }
    seen.add(dedupeKey)

    if (existingImages.has(dedupeKey)) {
      continue
    }

    const imageKey = remoteShareImageKeyForDedupeKey(dedupeKey)
    uploadTargets.push({
      workId: work.workId,
      dedupeKey,
      imageKey,
      uploadUrl: await createR2PresignedPutUrl(env, imageKey, work.thumbnail.contentType),
      contentType: work.thumbnail.contentType,
    })
  }

  return Response.json({
    deviceId,
    uploadTargets,
  })
}

async function handleCommitSyncWorks(
  request: Request,
  env: Env,
  deviceId: string,
): Promise<Response> {
  const body = deviceWorksSyncCommitInputSchema.parse(await request.json())
  const secretHash = await sha256Hex(body.deviceSecret)
  const device = await findDeviceById(env.DB, deviceId)

  if (!device) {
    notFound('device not found')
  }

  if (device.secretHash !== secretHash) {
    unauthorized('deviceSecret is invalid')
  }

  const uploadedMap = new Map<string, string>()
  for (const uploadedImage of body.uploadedImages) {
    uploadedMap.set(uploadedImage.dedupeKey, uploadedImage.imageKey)
    await insertRemoteShareImage(env.DB, uploadedImage.dedupeKey, uploadedImage.imageKey)
  }

  const missingDedupeKeys = body.works
    .map(work => remoteShareDedupeKeyForWork(work))
    .filter(dedupeKey => !uploadedMap.has(dedupeKey))
  const imageKeys = new Map(uploadedMap)
  const existingImages = await listRemoteShareImagesByDedupeKeys(env.DB, missingDedupeKeys)
  for (const [dedupeKey, imageKey] of existingImages) {
    imageKeys.set(dedupeKey, imageKey)
  }

  const syncedAt = new Date().toISOString()
  const existingRows = await listDeviceSnapshots(env.DB, deviceId)
  const existingMap = new Map(existingRows.map(row => [row.workId, row]))

  const worksToUpsert = body.works.filter(work => {
    const dedupeKey = remoteShareDedupeKeyForWork(work)
    const nextImageKey = imageKeys.get(dedupeKey) ?? null
    const nextTitle = work.title
    const nextEroId = work.erogamescapeId ?? null
    const nextOfficialUrl = work.officialUrl ?? null
    const nextErogamescapeUrl = work.erogamescapeUrl ?? null
    const nextSeiyaUrl = work.seiyaUrl ?? null
    const nextWidth = work.thumbnail?.width ?? null
    const nextHeight = work.thumbnail?.height ?? null

    const existing = existingMap.get(work.workId)
    if (!existing) {
      return true
    }
    
    return existing.title !== nextTitle ||
      existing.erogamescapeId !== nextEroId ||
      existing.officialUrl !== nextOfficialUrl ||
      existing.erogamescapeUrl !== nextErogamescapeUrl ||
      existing.seiyaUrl !== nextSeiyaUrl ||
      existing.imageKey !== nextImageKey ||
      existing.thumbnailWidth !== nextWidth ||
      existing.thumbnailHeight !== nextHeight
  })

  await upsertDeviceSnapshots(env.DB, deviceId, worksToUpsert, imageKeys, syncedAt)

  return Response.json({
    deviceId,
    syncedCount: body.works.length,
    lastSyncedAt: syncedAt,
  })
}

async function handleImageRequest(request: Request, env: Env, url: URL): Promise<Response> {
  const parts = url.pathname.split('/')
  const deviceId = parts[3]
  const imageKey = decodeURIComponent(parts.slice(5).join('/'))

  await requireSession(request, env.SESSION_SECRET, deviceId)

  if (env.R2_CUSTOM_ENDPOINT) {
    const bucketName = env.R2_BUCKET_NAME
    const minioUrl = `${env.R2_CUSTOM_ENDPOINT.replace(/\/$/, '')}/${bucketName}/${encodeObjectKey(imageKey)}`
    const upstreamResponse = await fetch(minioUrl)
    
    if (!upstreamResponse.ok) {
        return new Response('Not found', { status: 404 })
    }
    const headers = new Headers(upstreamResponse.headers)
    headers.set('Cache-Control', 'private, max-age=300')
    return new Response(upstreamResponse.body, { headers })
  }

  const object = await env.IMAGES.get(imageKey)
  if (!object) {
    return new Response('Not found', { status: 404 })
  }

  const headers = new Headers()
  object.writeHttpMetadata(headers)
  headers.set('Cache-Control', 'private, max-age=300')

  return new Response(object.body, {
    headers,
  })
}

async function handleLaunchBrokerConnect(
  request: Request,
  env: Env,
  deviceId: string,
  url: URL,
): Promise<Response> {
  const brokerId = env.REMOTE_LAUNCH_BROKER.idFromName(deviceId)
  const stub = env.REMOTE_LAUNCH_BROKER.get(brokerId)
  const proxyUrl = new URL(request.url)
  proxyUrl.pathname = `/connect`
  proxyUrl.searchParams.set('deviceId', deviceId)
  proxyUrl.searchParams.set('deviceSecret', url.searchParams.get('deviceSecret') ?? '')

  return stub.fetch(new Request(proxyUrl, request))
}

async function handleLaunchWork(
  request: Request,
  env: Env,
  deviceId: string,
  workId: string,
): Promise<Response> {
  const device = await findDeviceById(env.DB, deviceId)
  if (!device) {
    notFound('device not found')
  }

  await requireSession(request, env.SESSION_SECRET, deviceId)

  const brokerId = env.REMOTE_LAUNCH_BROKER.idFromName(deviceId)
  const stub = env.REMOTE_LAUNCH_BROKER.get(brokerId)
  const response = await stub.fetch('https://remote-launch-broker/request-launch', {
    method: 'POST',
    body: JSON.stringify({ workId }),
    headers: {
      'Content-Type': 'application/json',
    },
  })

  if (response.status === 400) {
    badRequest(await response.text())
  }

  if (response.status === 409) {
    return new Response(await response.text(), { status: 409 })
  }

  if (response.status !== 202) {
    return new Response(await response.text(), { status: 502 })
  }

  return new Response(null, { status: 202 })
}
