import type { RouteConfig } from '@mateothegreat/svelte5-router'
import type { TabPolicy } from '@/store/tabs/schema'

// 共有スキーマ型

export interface Descriptor<TComponent = any> {
  kind: string
  pathTemplate: string
  component: TComponent
  icon?: string
  tab: TabPolicy
}

// タブ関連の Extractor/ヘルパは src/store/tabs/schema.ts へ移動

// RouteConfig ビルド
export function buildRoutes(registry: readonly Descriptor[]): RouteConfig[] {
  return registry.map(d => ({
    path: toRouterPath(d.pathTemplate),
    component: d.component,
  }))
}

// タブ動作用: ロケーションからアクションを推定
// タブ用の補助機能は src/store/tabs/schema.ts へ移動

function toRouterPath(template: string): string {
  if (template.includes('(?<'))
    return template
  let out = template
  out = out.replace(/:([A-Z_]\w*)\(([^)]+)\)/gi, (_m, name, re) => `(?<${name}>${re})`)
  out = out.replace(/:([A-Z_]\w*)/gi, (_m, name) => `(?<${name}>[^/]+)`)
  return out
}
