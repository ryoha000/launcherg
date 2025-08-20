import type { CallbackSendNativeMessage } from '../../../test/setup/chrome'
import { create } from '@bufbuild/protobuf'
import {
  GetStatusRequestSchema,
  NativeMessageSchema,
  SyncStatusSchema,
} from '@launcherg/shared/proto/native_messaging'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { sendNativeMessageMock, setChromeRuntimeLastError } from '../../../test/setup/chrome'
import { createNativeMessenger } from './send'

describe('ネイティブメッセンジャー送受信とデコード', () => {
  const host = 'com.example.native'

  beforeEach(() => {
    vi.restoreAllMocks()
    sendNativeMessageMock.mockReset()
    setChromeRuntimeLastError(undefined)
  })

  it('buf oneof 形式のレスポンスを正しくデコードする', async () => {
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-1',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    const status = create(SyncStatusSchema, {
      totalSynced: 1,
      connectedExtensions: ['ext-1'],
      isRunning: true,
      connectionStatus: 1,
      errorMessage: '',
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, cb) => {
      cb?.({
        success: true,
        error: '',
        requestId: 'rid-1',
        response: { case: 'statusResult', value: status },
      })
    }) as CallbackSendNativeMessage)

    const res = await messenger.send(msg)
    expect(res).not.toBeNull()
    expect(res?.success).toBe(true)
    expect(res?.requestId).toBe('rid-1')
    expect(res?.response.case).toBe('statusResult')
    if (res?.response.case !== 'statusResult')
      throw new Error('unexpected case')
    expect(res.response.value.totalSynced).toBe(1)
  })

  it('cd ../', async () => {
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-2',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    const status = create(SyncStatusSchema, {
      totalSynced: 2,
      connectedExtensions: [],
      isRunning: false,
      connectionStatus: 1,
      errorMessage: '',
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, cb) => {
      cb?.({
        success: true,
        error: '',
        requestId: 'rid-2',
        response: { statusResult: status },
      })
    }) as CallbackSendNativeMessage)

    const res = await messenger.send(msg)
    expect(res).not.toBeNull()
    expect(res?.success).toBe(true)
    expect(res?.requestId).toBe('rid-2')
    expect(res?.response.case).toBe('statusResult')
    if (res?.response.case !== 'statusResult')
      throw new Error('unexpected case')
    expect(res.response.value.totalSynced).toBe(2)
  })

  it('chrome.runtime.lastError が設定されている場合は reject する', async () => {
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-3',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, cb) => {
      setChromeRuntimeLastError({ message: 'boom' })
      cb?.(null)
    }) as CallbackSendNativeMessage)

    await expect(messenger.send(msg)).rejects.toThrow('boom')
    setChromeRuntimeLastError(undefined)
  })

  it('null レスポンスは resolve(null) を返す', async () => {
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-4',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, cb) => {
      cb?.(null)
    }) as CallbackSendNativeMessage)

    const res = await messenger.send(msg)
    expect(res).toBeNull()
  })

  it('デコード不能なレスポンスは reject する', async () => {
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-5',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, cb) => {
      cb?.('not-a-json-object')
    }) as CallbackSendNativeMessage)

    await expect(messenger.send(msg)).rejects.toBeTruthy()
  })

  it('タイムアウトで reject する', async () => {
    vi.useFakeTimers()
    const messenger = createNativeMessenger(host)
    const msg = create(NativeMessageSchema, {
      requestId: 'rid-6',
      message: { case: 'getStatus', value: create(GetStatusRequestSchema, {}) },
    })

    sendNativeMessageMock.mockImplementation(((_host, _payload, _cb) => {
      // 応答しない
    }) as CallbackSendNativeMessage)

    const p = messenger.send(msg)
    await vi.advanceTimersByTimeAsync(30000)
    await expect(p).rejects.toThrow('timeout')
    vi.useRealTimers()
  })
})
