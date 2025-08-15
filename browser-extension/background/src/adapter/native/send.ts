import { fromJson, toJson, toJsonString } from '@bufbuild/protobuf'
import { logger } from '@launcherg/shared'
import { NativeMessageSchema, NativeResponseSchema } from '@launcherg/shared/proto/native_messaging'

const log = logger('background:native')

export function createNativeMessenger(nativeHostName: string) {
  return {
    async send(message: any): Promise<any | null> {
      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Native messaging timeout'))
        }, 30000)

        const jsonString = toJsonString(NativeMessageSchema, message)
        log.debug('Sending native message:', jsonString, toJson(NativeMessageSchema, message))

        chrome.runtime.sendNativeMessage(
          nativeHostName,
          // @ts-expect-error nullになりえるらしいがいったん無視
          toJson(NativeMessageSchema, message),
          (response) => {
            clearTimeout(timeout)

            if (chrome.runtime.lastError) {
              reject(new Error(chrome.runtime.lastError.message))
            }
            else if (response) {
              try {
                const nativeResponse = fromJson(NativeResponseSchema, response)
                log.debug('Received native response:', nativeResponse)
                try {
                  const responseJson = toJson(NativeResponseSchema, nativeResponse)
                  log.info('Native host response (json):', JSON.stringify(responseJson))
                }
                catch (jsonErr) {
                  log.warn('Failed to serialize native response to JSON:', jsonErr)
                }
                resolve(nativeResponse)
              }
              catch (e) {
                log.error('Failed to parse JSON response:', e)
                reject(e)
              }
            }
            else {
              resolve(null)
            }
          },
        )
      })
    },
  }
}

export {}
