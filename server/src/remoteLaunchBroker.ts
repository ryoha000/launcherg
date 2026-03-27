import type { Env } from '@server/env'

import { findDeviceById } from '@server/lib/db'
import {
  launchWorkRequestSchema,
  launchWorkResponseSchema,
  launchWorkTransportAckSchema,
  type LaunchReasonCode,
  type LaunchWorkResponse,
} from '@server/shared/schema'
import { sha256Hex } from '@server/lib/crypto'
import { unauthorized } from '@server/lib/errors'
import { DurableObject } from 'cloudflare:workers'

interface DesktopAttachment {
  kind: 'desktop'
  deviceId: string
}

interface PendingLaunch {
  resolve: (response: LaunchWorkResponse) => void
  timeoutId: ReturnType<typeof setTimeout>
}

const LAUNCH_TIMEOUT_MS = 15_000

const LAUNCH_MESSAGES: Record<LaunchReasonCode, string> = {
  launched: 'デスクトップアプリでゲームを起動しました。',
  desktop_not_connected: 'デスクトップアプリが Remote Launch に接続していません。Launcherg を起動した状態で再試行してください。',
  timeout: 'デスクトップアプリから起動結果を受信できませんでした。しばらくしてから再試行してください。',
  work_not_found: 'この作品は現在の PC に同期されていません。ライブラリを再同期してください。',
  shortcut_not_found: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
  target_not_installed: '起動先のファイルが見つかりませんでした。ゲームが未インストールか、インストール先が移動された可能性があります。',
  launch_failed: 'ゲームの起動に失敗しました。PC 側で直接起動できるか確認してください。',
  unknown: 'ゲームを起動できませんでした。しばらくしてから再試行してください。',
}

function isDesktopAttachment(value: unknown): value is DesktopAttachment {
  if (!value || typeof value !== 'object') {
    return false
  }

  const entry = value as Partial<DesktopAttachment>
  return entry.kind === 'desktop' && typeof entry.deviceId === 'string'
}

function createLaunchWorkResponse(
  ok: boolean,
  reasonCode: LaunchReasonCode,
  detail: string | null = null,
): LaunchWorkResponse {
  return {
    ok,
    reasonCode,
    message: LAUNCH_MESSAGES[reasonCode],
    detail,
  }
}

function createDesktopNotConnectedResponse(detail = 'desktop websocket is not connected'): LaunchWorkResponse {
  return createLaunchWorkResponse(false, 'desktop_not_connected', detail)
}

function createTimeoutResponse(): LaunchWorkResponse {
  return createLaunchWorkResponse(false, 'timeout', 'launch ack timed out')
}

function createUnknownLaunchResponse(detail: string): LaunchWorkResponse {
  return createLaunchWorkResponse(false, 'unknown', detail)
}

export class RemoteLaunchBrokerState {
  private readonly pendingLaunches = new Map<string, PendingLaunch>()

  getPendingLaunchCount(): number {
    return this.pendingLaunches.size
  }

  requestLaunch(sockets: WebSocket[], workId: string): Promise<LaunchWorkResponse> {
    const parsedWorkId = launchWorkRequestSchema.safeParse({ workId })
    if (!parsedWorkId.success) {
      return Promise.resolve(createUnknownLaunchResponse('workId is required'))
    }

    const desktopSocket = sockets.find(socket => isDesktopAttachment(socket.deserializeAttachment()))
    if (!desktopSocket) {
      return Promise.resolve(createDesktopNotConnectedResponse())
    }

    const requestId = crypto.randomUUID()
    const payload = {
      type: 'launch-work',
      requestId,
      workId: parsedWorkId.data.workId,
    }

    return new Promise<LaunchWorkResponse>((resolve) => {
      const timeoutId = setTimeout(() => {
        this.completeLaunch(requestId, createTimeoutResponse())
      }, LAUNCH_TIMEOUT_MS)

      this.pendingLaunches.set(requestId, {
        resolve,
        timeoutId,
      })

      try {
        desktopSocket.send(JSON.stringify(payload))
      }
      catch (error) {
        this.completeLaunch(requestId, createDesktopNotConnectedResponse(String(error)))
      }
    })
  }

  handleWebSocketMessage(_ws: WebSocket, message: string | ArrayBuffer): void {
    if (typeof message !== 'string') {
      return
    }

    let payload: unknown
    try {
      payload = JSON.parse(message)
    }
    catch {
      return
    }

    const parsed = launchWorkTransportAckSchema.safeParse(payload)
    if (!parsed.success) {
      return
    }

    const pending = this.pendingLaunches.get(parsed.data.requestId)
    if (!pending) {
      return
    }

    const response = launchWorkResponseSchema.parse({
      ok: parsed.data.ok,
      reasonCode: parsed.data.reasonCode,
      message: parsed.data.message,
      detail: parsed.data.detail,
    })
    this.completeLaunch(parsed.data.requestId, response)
  }

  private completeLaunch(requestId: string, response: LaunchWorkResponse): void {
    const pending = this.pendingLaunches.get(requestId)
    if (!pending) {
      return
    }

    clearTimeout(pending.timeoutId)
    this.pendingLaunches.delete(requestId)
    pending.resolve(response)
  }
}

export class RemoteLaunchBroker extends DurableObject<Env> {
  private readonly state = new RemoteLaunchBrokerState()

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

  override webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): void {
    this.state.handleWebSocketMessage(ws, message)
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
    const parsed = launchWorkRequestSchema.safeParse({
      workId: payload.workId ?? '',
    })

    if (!parsed.success) {
      return Response.json(createUnknownLaunchResponse('workId is required'))
    }

    const response = await this.state.requestLaunch(this.ctx.getWebSockets(), parsed.data.workId)
    return Response.json(response)
  }
}

export async function requestLaunch(
  broker: DurableObjectStub,
  workId: string,
): Promise<LaunchWorkResponse> {
  const response = await broker.fetch('https://remote-launch-broker/request-launch', {
    method: 'POST',
    body: JSON.stringify(launchWorkRequestSchema.parse({ workId })),
    headers: {
      'Content-Type': 'application/json',
    },
  })

  return launchWorkResponseSchema.parse(await response.json())
}
