import type { Env } from '@server/env'
import { handleRequest } from '@server/app'
import { RemoteLaunchBroker } from '@server/remoteLaunchBroker'

export default {
  fetch(request: Request, env: Env): Promise<Response> {
    return handleRequest(request, env)
  },
}

export { RemoteLaunchBroker }
