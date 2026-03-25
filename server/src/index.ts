import type { Env } from '@server/env'
import { handleRequest } from '@server/app'

export default {
  fetch(request: Request, env: Env): Promise<Response> {
    return handleRequest(request, env)
  },
}
