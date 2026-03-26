import type { Env } from '@server/env'

import { findDeviceById } from '@server/lib/db'
import { sha256Hex } from '@server/lib/crypto'
import { unauthorized } from '@server/lib/errors'
import { DurableObject } from 'cloudflare:workers'

interface DesktopAttachment {
  kind: 'desktop'
  deviceId: string
}

interface LaunchRequestMessage {
  type: 'launch-work'
  workId: string
}

function isDesktopAttachment(value: unknown): value is DesktopAttachment {
  if (!value || typeof value !== 'object') {
    return false
  }

  const entry = value as Partial<DesktopAttachment>
  return entry.kind === 'desktop' && typeof entry.deviceId === 'string'
}

export class RemoteLaunchBroker extends DurableObject<Env> {
  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url)

    if (url.pathname.endsWith('/connect')) {
      return this.handleConnect(request, url)
    }

    if (url.pathname.endsWith('/request-launch') && request.method === 'POST') {
      return this.handleRequestLaunch(request)
    }

    return new Response('Not found', { status: 404 })
  }

  override webSocketMessage(_ws: WebSocket, _message: string | ArrayBuffer): void {
  }

  private async handleConnect(request: Request, url: URL): Promise<Response> {
    const upgradeHeader = request.headers.get('Upgrade')
    if (upgradeHeader?.toLowerCase() !== 'websocket') {
      return new Response('Expected websocket', { status: 426 })
    }

    const deviceId = url.searchParams.get('deviceId') ?? ''
    const deviceSecret = url.searchParams.get('deviceSecret') ?? ''
    const device = await findDeviceById(this.env.DB, deviceId)
    const secretHash = await sha256Hex(deviceSecret)

    if (!device || device.secretHash !== secretHash) {
      unauthorized('deviceSecret is invalid')
    }

    const pair = new WebSocketPair()
    const [client, server] = Object.values(pair)
    this.ctx.acceptWebSocket(server)
    server.serializeAttachment({
      kind: 'desktop',
      deviceId,
    } satisfies DesktopAttachment)

    return new Response(null, {
      status: 101,
      webSocket: client,
    })
  }

  private async handleRequestLaunch(request: Request): Promise<Response> {
    const payload = await request.json() as { workId?: string }
    const workId = payload.workId?.trim() ?? ''

    if (!workId) {
      return new Response('workId is required', { status: 400 })
    }

    const sockets = this.ctx.getWebSockets().filter((socket) => {
      const attachment = socket.deserializeAttachment()
      return isDesktopAttachment(attachment)
    })

    if (sockets.length === 0) {
      return new Response('Desktop is not connected', { status: 409 })
    }

    const message: LaunchRequestMessage = {
      type: 'launch-work',
      workId,
    }

    for (const socket of sockets) {
      socket.send(JSON.stringify(message))
    }

    return new Response(null, { status: 202 })
  }
}
