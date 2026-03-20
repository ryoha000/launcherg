const INSTALL_FLAG = '__launchergDlsiteNetworkHookInstalled__'
const DLSITE_HOOK_MESSAGE_SOURCE = 'launcherg'
const DLSITE_WORKS_MESSAGE_TYPE = 'launcherg:dlsite-works-response'
const DLSITE_WORKS_HOST = 'play.dlsite.com'
const DLSITE_WORKS_PATH = '/api/v3/content/works'

function isDlsiteWorksApiUrl(url: string): boolean {
  try {
    const parsed = new URL(url, `https://${DLSITE_WORKS_HOST}`)
    return parsed.hostname === DLSITE_WORKS_HOST && parsed.pathname === DLSITE_WORKS_PATH
  }
  catch {
    return false
  }
}

function isDlsiteWorksResponse(value: unknown): boolean {
  return typeof value === 'object' && value !== null && 'works' in value
}

function postPayload(requestUrl: string, payload: unknown): void {
  if (!isDlsiteWorksResponse(payload))
    return

  window.postMessage({
    source: DLSITE_HOOK_MESSAGE_SOURCE,
    type: DLSITE_WORKS_MESSAGE_TYPE,
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

    if (isDlsiteWorksApiUrl(requestUrl)) {
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
      if (!isDlsiteWorksApiUrl(requestUrl))
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
  installFetchHook()
  installXhrHook()
}

install()
