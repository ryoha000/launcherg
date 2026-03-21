import type { RouteConfig } from '@mateothegreat/svelte5-router'
import { ROUTE_REGISTRY } from './const'
import { buildRoutes } from './registry'

export const routes: RouteConfig[] = buildRoutes(ROUTE_REGISTRY)
