import { logger } from '@launcherg/shared'

const log = logger('background:native')

function isObjectRecord(value: unknown): value is object {
  return typeof value === 'object' && value !== null
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

export function createNativeMessenger(nativeHostName: string) {
  const TIMEOUT_MS = 30000

  const sendJson = async <TRes = unknown>(message: object): Promise<TRes | null> => {
    return new Promise((_resolve, _reject) => {
      const { resolveOnce: resolve, rejectOnce: reject, startTimer } = createOnceSettled(_resolve, _reject)
      startTimer(TIMEOUT_MS, () => new Error('Native messaging timeout'))

      if (!isObjectRecord(message))
        return reject(new Error('Encoded JSON message is not an object'))

      const requestId = (message as any).request_id ?? (message as any).requestId
      log.debug('Sending native message(JSON)', { host: nativeHostName, requestId, message })

      const startTime = Date.now()
      chrome.runtime.sendNativeMessage(
        nativeHostName,
        message,
        (response) => {
          if (chrome.runtime.lastError)
            return reject(new Error(chrome.runtime.lastError.message))
          log.debug('Received native message(JSON)', { ellapsed: Date.now() - startTime, host: nativeHostName, response })
          resolve(response as TRes)
        },
      )
    })
  }

  return { sendJson }
}

export {}
