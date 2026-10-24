export {}

;(() => {
  const INSTALL_FLAG = '__launchergDmmNetworkHookInstalled__'
  const INSTALL_MARKER = 'data-launcherg-dmm-network-hook-installed'
  const DMM_HOOK_MESSAGE_SOURCE = 'launcherg'
  const DMM_LIBRARY_MESSAGE_TYPE = 'launcherg:dmm-library-response'
  const DMM_LIBRARY_HOST = 'dlsoft.dmm.co.jp'
  const DMM_LIBRARY_PATH = '/ajax/v1/library'

  let contentReady = false
  const pendingPayloads: object[] = []
  function reportHookStatus(stage: string, detail = ''): void {
    window.postMessage({ source: 'launcherg', type: 'launcherg:dmm-hook-status', stage, detail }, window.location.origin)
  }
  function deliver(payload: object): void {
    if (contentReady) {
      window.postMessage(payload, window.location.origin)
    }
    else {
      pendingPayloads.push(payload)
      if (pendingPayloads.length > 20)
        pendingPayloads.shift()
    }
  }

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
    if (!isDmmLibraryResponse(payload)) {
      reportHookStatus('api_invalid', '購入一覧のレスポンス形式が未対応です')
      return
    }

    deliver({
      source: DMM_HOOK_MESSAGE_SOURCE,
      type: DMM_LIBRARY_MESSAGE_TYPE,
      pageUrl: window.location.href,
      requestUrl,
      payload,
    })
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
      const [input] = args
      const requestUrl = typeof input === 'string' ? input : input instanceof Request ? input.url : String(input)

      if (isDmmLibraryApiUrl(requestUrl))
        reportHookStatus('api_requested')
      let response: Response
      try {
        response = await originalFetch(...args)
      }
      catch (error) {
        if (isDmmLibraryApiUrl(requestUrl))
          reportHookStatus('api_failed', '購入一覧APIの通信に失敗しました')
        throw error
      }

      if (isDmmLibraryApiUrl(requestUrl)) {
        reportHookStatus('api_response', `HTTP ${response.status}`)
        void response.clone().text().then((text) => {
          postPayload(requestUrl, parseJsonText(text))
        }).catch(() => reportHookStatus('api_invalid', '購入一覧のレスポンスを読み取れませんでした'))
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

        reportHookStatus('api_response', `HTTP ${this.status}`)
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
    window.addEventListener('message', (event: MessageEvent<unknown>) => {
      if (event.source !== window || typeof event.data !== 'object' || event.data === null)
        return
      const data = event.data as { source?: string, type?: string }
      if (data.source !== 'launcherg' || data.type !== 'launcherg:dmm-content-ready')
        return
      contentReady = true
      reportHookStatus('hook_ready')
      for (const payload of pendingPayloads.splice(0))
        window.postMessage(payload, window.location.origin)
    })
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
})()
