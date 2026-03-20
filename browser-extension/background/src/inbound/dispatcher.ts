import type { ExtensionRequest, ExtensionResponse } from '@launcherg/shared'
import type { HandlerContext } from '../shared/types'
import { logger } from '@launcherg/shared'
import { handleDebugNativeMessage, handleGetStatus, handleSyncDlsiteGames, handleSyncDmmGames } from '../usecase'

const log = logger('background:dispatcher')

export function createMessageDispatcher(context: HandlerContext) {
  return async function dispatch(message: unknown): Promise<unknown> {
    try {
      const extensionRequest = message as ExtensionRequest

      switch (extensionRequest.request.case) {
        case 'syncDmmGames':
          return await handleSyncDmmGames(
            context,
            extensionRequest.requestId,
            extensionRequest.request.value,
          )
        case 'syncDlsiteGames':
          return await handleSyncDlsiteGames(
            context,
            extensionRequest.requestId,
            extensionRequest.request.value,
          )
        case 'getStatus':
          return await handleGetStatus(
            context,
            extensionRequest.requestId,
            extensionRequest.request.value,
          )
        case 'debugNativeMessage':
          return await handleDebugNativeMessage(
            context,
            extensionRequest.requestId,
            extensionRequest.request.value,
          )
        default: {
          log.warn('Unknown request type:', extensionRequest.request.case)
          const res: ExtensionResponse = {
            requestId: extensionRequest.requestId,
            success: false,
            error: 'Unknown request type',
            response: { case: undefined },
          }
          return res
        }
      }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      log.error('Error handling message:', errorMessage)
      const res: ExtensionResponse = {
        requestId: (typeof message === 'object' && message !== null && 'requestId' in message && typeof (message as any).requestId === 'string')
          ? (message as any).requestId
          : 'unknown',
        success: false,
        error: errorMessage,
        response: { case: undefined },
      }
      return res
    }
  }
}

export { createMessageDispatcher as createMessageHandler }
