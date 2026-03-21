const INSTALL_FLAG = '__launchergDmmNetworkHookInstalled__'
const INSTALL_MARKER = 'data-launcherg-dmm-network-hook-installed'
const DMM_HOOK_MESSAGE_SOURCE = 'launcherg'
const DMM_LIBRARY_MESSAGE_TYPE = 'launcherg:dmm-library-response'
const DMM_LIBRARY_HOST = 'dlsoft.dmm.co.jp'
const DMM_LIBRARY_PATH = '/ajax/v1/library'

function isDmmLibraryApiUrl(url: string): boolean {
  try {
    const parsed = new URL(url, `https://${DMM_LIBRARY_HOST}`)
    return parsed.hostname === DMM_LIBRARY_HOST && parsed.pathname === DMM_LIBRARY_PATH
  }
  catch {
    return false
  }
}

function isDmmLibraryResponse(value: unknown): boolean {
  return typeof value === 'object' && value !== null && 'error' in value
}

function postPayload(requestUrl: string, payload: unknown): void {
  if (!isDmmLibraryResponse(payload))
    return

  window.postMessage({
    source: DMM_HOOK_MESSAGE_SOURCE,
    type: DMM_LIBRARY_MESSAGE_TYPE,
    pageUrl: window.location.href,
    requestUrl,
    payload,
  }, window.location.origin)
}

function parseJsonText(text: string): unknown {
  try {
    return JSON.parse(text)
  }
  catch {
    return null
  }
}

function installFetchHook(): void {
  const originalFetch = window.fetch.bind(window)
  window.fetch = async (...args) => {
    const response = await originalFetch(...args)
    const [input] = args
    const requestUrl = typeof input === 'string' ? input : input instanceof Request ? input.url : String(input)

    if (isDmmLibraryApiUrl(requestUrl)) {
      void response.clone().text().then((text) => {
        postPayload(requestUrl, parseJsonText(text))
      }).catch(() => {})
    }

    return response
  }
}

function installXhrHook(): void {
  const originalOpen = XMLHttpRequest.prototype.open
  const originalSend = XMLHttpRequest.prototype.send

  XMLHttpRequest.prototype.open = function open(method: string, url: string | URL, ...rest: [boolean?, string?, string?]) {
    ;(this as XMLHttpRequest & { __launchergRequestUrl?: string }).__launchergRequestUrl = String(url)
    return originalOpen.apply(this, [method, url, ...rest] as Parameters<XMLHttpRequest['open']>)
  }

  XMLHttpRequest.prototype.send = function send(...args: Parameters<XMLHttpRequest['send']>) {
    this.addEventListener('load', () => {
      const requestUrl = (this as XMLHttpRequest & { __launchergRequestUrl?: string }).__launchergRequestUrl || ''
      if (!isDmmLibraryApiUrl(requestUrl))
        return

      const payload = this.responseType === 'json'
        ? this.response
        : parseJsonText(this.responseText)

      postPayload(requestUrl, payload)
    })

    return originalSend.apply(this, args)
  }
}

function install(): void {
  const globalWindow = window as typeof window & { [INSTALL_FLAG]?: boolean }
  if (globalWindow[INSTALL_FLAG])
    return

  globalWindow[INSTALL_FLAG] = true
  const setMarker = (): void => {
    document.documentElement?.setAttribute(INSTALL_MARKER, 'true')
  }
  if (document.documentElement)
    setMarker()
  else
    window.addEventListener('DOMContentLoaded', setMarker, { once: true })
  installFetchHook()
  installXhrHook()
}

install()
