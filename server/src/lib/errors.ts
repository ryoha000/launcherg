import { ORPCError } from '@orpc/server'

export function badRequest(message: string): never {
  throw new ORPCError('BAD_REQUEST', { message })
}

export function unauthorized(message = 'Unauthorized'): never {
  throw new ORPCError('UNAUTHORIZED', { message })
}

export function notFound(message = 'Not found'): never {
  throw new ORPCError('NOT_FOUND', { message })
}

export function internalServerError(message = 'Internal Server Error'): never {
  throw new ORPCError('INTERNAL_SERVER_ERROR', { message })
}
