import type { Descriptor } from '@/router/registry'

// タブ関連スキーマ
export type Extractor<T> = (input: {
  path: string
  pathParams?: Record<string, unknown>
  queryParams?: Record<string, unknown>
}) => T | undefined

export type TabPolicy
  = | { mode: 'none' }
    | { mode: 'singleton', title: string }
    | {
      mode: 'keyed'
      key: Extractor<string>
      title?: Extractor<string>
    }

export function pathParamExtractor(name: string): Extractor<string> {
  return ({ pathParams }) => {
    const value = pathParams?.[name]
    if (typeof value === 'string')
      return value
    if (typeof value === 'number')
      return String(value)
    return undefined
  }
}

export function queryParamExtractor(
  name: string,
  editor?: (value: string, ctx: {
    path: string
    pathParams?: Record<string, unknown>
    queryParams?: Record<string, unknown>
  }) => string,
): Extractor<string> {
  return (ctx) => {
    const raw = ctx.queryParams?.[name]
    if (typeof raw === 'string') {
      const decoded = safeDecodeURIComponent(raw)
      return editor ? editor(decoded, ctx) : decoded
    }
    return undefined
  }
}

function safeDecodeURIComponent(src: string): string {
  try {
    return decodeURIComponent(src)
  }
  catch {
    return src
  }
}

export function singletonTab(title: string): TabPolicy {
  return { mode: 'singleton', title }
}

export function keyedTab(
  keyExtractor: Extractor<string>,
  titleExtractor?: Extractor<string>,
): TabPolicy {
  return { mode: 'keyed', key: keyExtractor, title: titleExtractor }
}

// タブ動作用: ロケーションからアクションを推定
export type TabAction
  = | { mode: 'none' }
    | { mode: 'singleton', type: string, title: string }
    | { mode: 'keyed', type: string, key: string, title?: string, href: string }

export function getTabActionFromLocation(
  registry: readonly Descriptor[],
  input: {
    path: string
    pathParams?: Record<string, unknown>
    queryParams?: Record<string, unknown>
    href?: string
  },
): TabAction {
  const matched = matchByTemplate(registry, input.path)
  if (!matched)
    return { mode: 'none' }

  const { descriptor } = matched
  const tab = descriptor.tab
  switch (tab.mode) {
    case 'none': {
      return { mode: 'none' }
    }
    case 'singleton': {
      return { mode: 'singleton', type: descriptor.kind, title: tab.title }
    }
    case 'keyed': {
      const key = tab.key({
        path: input.path,
        pathParams: matched.params,
        queryParams: input.queryParams,
      })
      const title = tab.title?.({
        path: input.path,
        pathParams: matched.params,
        queryParams: input.queryParams,
      })
      if (key === undefined || key === null)
        return { mode: 'none' }
      return {
        mode: 'keyed',
        type: descriptor.kind,
        key,
        title,
        href: input.href ?? buildHref(input.path, input.queryParams),
      }
    }
    default: {
      const _exhaustive: never = tab
      return _exhaustive
    }
  }
}

// URL 構築（タブ遷移用）
export function buildPath(
  descriptor: Descriptor,
  key?: string,
): string {
  const { pathTemplate } = descriptor
  const tab = descriptor.tab
  switch (tab.mode) {
    case 'none':
      return normalizeTemplate(pathTemplate)
    case 'singleton':
      return normalizeTemplate(pathTemplate)
    case 'keyed':
      if (key === undefined)
        return normalizeTemplate(pathTemplate)
      return fillTemplate(pathTemplate, { id: String(key) })
    default: {
      const _exhaustive: never = tab
      return _exhaustive
    }
  }
}

export function stripQueryParams(
  href: string,
  keys: string[],
): string {
  try {
    const url = new URL(href, 'http://launcherg.local')
    for (const key of keys) {
      url.searchParams.delete(key)
    }

    const search = url.searchParams.toString()
    return `${url.pathname}${search ? `?${search}` : ''}${url.hash}`
  }
  catch {
    return href
  }
}

// 内部: pathTemplate と path をマッチさせ簡易 params を得る
function matchByTemplate(
  registry: readonly Descriptor[],
  path: string,
): { descriptor: Descriptor, params: Record<string, string> } | null {
  for (const d of registry) {
    const m = compileTemplateToRegex(d.pathTemplate).exec(path)
    if (m) {
      const groups = m.groups ?? {}
      const params: Record<string, string> = {}
      for (const [k, v] of Object.entries(groups))
        params[k] = v as string
      return { descriptor: d, params }
    }
  }
  return null
}

function normalizeTemplate(template: string): string {
  return template.replace(/\(\?:\?<[^>]+>[^)]+\)/g, '')
}

function buildHref(path: string, queryParams?: Record<string, unknown>): string {
  if (!queryParams)
    return path

  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(queryParams)) {
    if (value === undefined || value === null)
      continue

    if (Array.isArray(value)) {
      for (const item of value)
        params.append(key, String(item))
      continue
    }

    params.append(key, String(value))
  }

  const querystring = params.toString()
  return querystring ? `${path}?${querystring}` : path
}

function compileTemplateToRegex(template: string): RegExp {
  // 例: /works/:id(\d+) → ^/works/(?<id>\d+)$
  let pattern = template
  pattern = pattern.replace(/:(\w+)\(([^)]+)\)/g, (_m, name, re) => `(?<${name}>${re})`)
  pattern = pattern.replace(/:(\w+)/g, (_m, name) => `(?<${name}>[^/]+)`)
  return new RegExp(`^${pattern}$`)
}

function fillTemplate(template: string, params: Record<string, string>): string {
  let out = template
  for (const [k, v] of Object.entries(params)) {
    out = out.replace(new RegExp(`:${k}\\([^)]*\\)`), v)
    out = out.replace(new RegExp(`:${k}(?!\\()`, 'g'), v)
  }
  return out
}
