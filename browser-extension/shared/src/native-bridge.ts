import type { ExtensionRequest, ExtensionResponse } from './proto/extension_internal'
import { fromJson } from '@bufbuild/protobuf'
import { ExtensionResponseSchema } from './proto/extension_internal'

export async function sendExtensionRequestRaw<T extends ExtensionRequest>(request: T, encode: (req: T) => unknown): Promise<ExtensionResponse> {
  return new Promise((resolve, reject) => {
    const payload = encode(request)
    chrome.runtime.sendMessage(payload, (response) => {
      if (chrome.runtime.lastError)
        return reject(new Error(chrome.runtime.lastError.message))
      try {
        const decoded = fromJson(ExtensionResponseSchema, response)
        resolve(decoded)
      }
      catch (e) {
        reject(e)
      }
    })
  })
}
