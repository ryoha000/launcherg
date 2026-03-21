import type { ExtensionRequest, ExtensionResponse } from './models'

export async function sendExtensionRequestRaw<T extends ExtensionRequest>(request: T, encode: (req: T) => unknown): Promise<ExtensionResponse> {
  return new Promise((resolve, reject) => {
    const payload = encode(request)
    chrome.runtime.sendMessage(payload, (response) => {
      if (chrome.runtime.lastError)
        return reject(new Error(chrome.runtime.lastError.message))
      try {
        resolve(response as ExtensionResponse)
      }
      catch (e) {
        reject(e)
      }
    })
  })
}

// Unified helper for ExtensionRequest without serializer
export async function sendExtensionRequest(request: ExtensionRequest): Promise<ExtensionResponse> {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(request, (response) => {
      const lastError = chrome.runtime?.lastError
      if (lastError)
        return reject(new Error(lastError.message))
      resolve(response as ExtensionResponse)
    })
  })
}

// Send a plain JSON object (already shaped) and get raw JSON response back
export async function sendJson<TReq extends object, TRes = unknown>(payload: TReq): Promise<TRes> {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(payload, (response) => {
      const lastError = chrome.runtime?.lastError
      if (lastError)
        return reject(new Error(lastError.message))
      resolve(response as TRes)
    })
  })
}
