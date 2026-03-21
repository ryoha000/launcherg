export const DEEP_LINK_SCHEME = 'launcherg'

export interface ParsedDeepLinkTarget {
  kind: string
  path: string
  key?: string
  play?: boolean
}

export function buildWorkOpenDeepLink(
  workId: string,
  options: { play?: boolean, gamename?: string } = {},
): string {
  const url = new URL(`${DEEP_LINK_SCHEME}://works/open`)
  url.searchParams.set('id', workId)
  if (options.gamename) {
    url.searchParams.set('gamename', options.gamename)
  }
  if (options.play) {
    url.searchParams.set('play', 'true')
  }
  return url.toString()
}

export function parseDeepLinkUrl(rawUrl: string): ParsedDeepLinkTarget | null {
  let url: URL
  try {
    url = new URL(rawUrl)
  }
  catch {
    return null
  }

  if (url.protocol.toLowerCase() !== `${DEEP_LINK_SCHEME}:`)
    return null

  switch (url.host.toLowerCase()) {
    case 'works': {
      if (normalizePathname(url.pathname) !== '/open')
        return null

      const key = url.searchParams.get('id')
      if (!key)
        return null

      const gamename = url.searchParams.get('gamename')
      const query = new URLSearchParams()
      if (gamename) {
        query.set('gamename', gamename)
      }
      if (url.searchParams.get('play') === 'true') {
        query.set('play', 'true')
      }

      return {
        kind: 'works',
        key,
        path: query.toString()
          ? `/works/${encodeURIComponent(key)}?${query.toString()}`
          : `/works/${encodeURIComponent(key)}`,
        play: url.searchParams.get('play') === 'true',
      }
    }
    case 'settings': {
      if (!isRootPath(url.pathname))
        return null

      return {
        kind: 'settings',
        path: '/settings',
      }
    }
    default:
      return null
  }
}

function normalizePathname(pathname: string): string {
  return pathname === '/' ? '' : pathname
}

function isRootPath(pathname: string): boolean {
  return normalizePathname(pathname) === ''
}
