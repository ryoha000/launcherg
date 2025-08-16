import type { NativeMessage, NativeResponse } from '@launcherg/shared/proto/native_messaging'
import { fromJson, toJson } from '@bufbuild/protobuf'
import { logger } from '@launcherg/shared'
import { NativeMessageSchema, NativeResponseSchema } from '@launcherg/shared/proto/native_messaging'

const log = logger('background:native')

function isNonNull<T>(value: T | null | undefined): value is T {
  return value != null
}

function isObjectRecord(value: unknown): value is object {
  return typeof value === 'object' && value !== null
}

function decodeNativeResponse(payload: unknown): NativeResponse {
  return fromJson(NativeResponseSchema, payload as any)
}

function createOnceSettled<T, E>(resolve: (value: T) => void, reject: (reason: E) => void) {
  let settled = false
  let timer: ReturnType<typeof setTimeout> | null = null

  const finalize = <A>(fn: (arg: A) => void) => (arg: A) => {
    if (settled)
      return
    settled = true
    if (timer)
      clearTimeout(timer)
    fn(arg)
  }

  const resolveOnce = finalize(resolve)
  const rejectOnce = finalize(reject)

  const startTimer = (ms: number, buildError: () => E) => {
    timer = setTimeout(() => rejectOnce(buildError()), ms)
  }

  return { resolveOnce, rejectOnce, startTimer }
}

function recieve(requestId: string, response: unknown, resolve: (value: NativeResponse | null) => void, reject: (reason?: any) => void): void {
  if (chrome.runtime.lastError) {
    reject(new Error(chrome.runtime.lastError.message))
  }

  if (response == null) {
    resolve(null)
    return
  }

  try {
    const nativeResponse = decodeNativeResponse(response)
    log.debug('Received native response', { requestId, success: nativeResponse.success })
    resolve(nativeResponse)
  }
  catch (e) {
    log.error('Failed to parse native response', e)
    reject(e)
  }
}

export function createNativeMessenger(nativeHostName: string) {
  const TIMEOUT_MS = 30000
  const send = async (message: NativeMessage): Promise<NativeResponse | null> => {
    return new Promise((_resolve, _reject) => {
      const { resolveOnce: resolve, rejectOnce: reject, startTimer } = createOnceSettled(_resolve, _reject)
      startTimer(TIMEOUT_MS, () => new Error('Native messaging timeout'))

      const payload = toJson(NativeMessageSchema, message)
      if (!isNonNull(payload))
        return reject(new Error('Failed to encode native message'))
      if (!isObjectRecord(payload))
        return reject(new Error('Encoded native message is not an object'))

      log.debug('Sending native message', { host: nativeHostName, requestId: message.requestId })

      chrome.runtime.sendNativeMessage(
        nativeHostName,
        payload,
        (response) => {
          recieve(message.requestId, response, resolve, reject)
        },
      )
    })
  }

  return { send }
}

export {}
