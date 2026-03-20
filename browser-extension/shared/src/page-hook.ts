export function injectPageScript(scriptUrl: string, markerId: string): boolean {
  if (document.getElementById(markerId))
    return false

  const parent = document.documentElement || document.head || document.body
  if (!parent)
    return false

  const script = document.createElement('script')
  script.id = markerId
  script.src = scriptUrl
  script.async = false
  script.dataset.launchergInjected = 'true'
  script.addEventListener('load', () => script.remove(), { once: true })
  script.addEventListener('error', () => script.remove(), { once: true })
  parent.appendChild(script)
  return true
}

export function isTypedWindowMessage<T extends { source: string, type: string }>(
  event: MessageEvent<unknown>,
  expectedSource: string,
  expectedType: string,
): event is MessageEvent<T> {
  return event.source === window
    && typeof event.data === 'object'
    && event.data !== null
    && 'source' in event.data
    && 'type' in event.data
    && (event.data as { source?: unknown }).source === expectedSource
    && (event.data as { type?: unknown }).type === expectedType
}

export interface UrlBoundPayloadCache<T> {
  get: () => T | null
  set: (payload: T) => void
  clear: () => void
  resetIfUrlChanged: (url: string) => boolean
}

export function createUrlBoundPayloadCache<T>(initialUrl: string): UrlBoundPayloadCache<T> {
  let currentUrl = initialUrl
  let payload: T | null = null

  return {
    get: () => payload,
    set: (nextPayload: T) => {
      payload = nextPayload
    },
    clear: () => {
      payload = null
    },
    resetIfUrlChanged: (url: string) => {
      if (url === currentUrl)
        return false
      currentUrl = url
      payload = null
      return true
    },
  }
}
