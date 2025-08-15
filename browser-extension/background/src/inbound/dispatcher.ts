import type { JsonValue } from '@bufbuild/protobuf'
import type { HandlerContext } from '../shared/types'
import { create, fromJson, toJson } from '@bufbuild/protobuf'
import { logger } from '@launcherg/shared'
import { ExtensionRequestSchema, ExtensionResponseSchema } from '@launcherg/shared/proto/extension_internal'
import { handleDebugNativeMessage, handleGetStatus, handleSyncDlsiteGames, handleSyncDmmGames } from '../usecase'

const log = logger('background:dispatcher')

export function createMessageDispatcher(context: HandlerContext) {
  return async function dispatch(message: unknown): Promise<any> {
    try {
      const extensionRequest = fromJson(ExtensionRequestSchema, message as JsonValue)

      switch (extensionRequest.request.case) {
        case 'syncDmmGames':
          return toJson(
            ExtensionResponseSchema,
            await handleSyncDmmGames(
              context,
              extensionRequest.requestId,
              extensionRequest.request.value,
            ),
          )
        case 'syncDlsiteGames':
          return toJson(
            ExtensionResponseSchema,
            await handleSyncDlsiteGames(
              context,
              extensionRequest.requestId,
              extensionRequest.request.value,
            ),
          )
        case 'getStatus':
          return toJson(
            ExtensionResponseSchema,
            await handleGetStatus(
              context,
              extensionRequest.requestId,
              extensionRequest.request.value,
            ),
          )
        case 'debugNativeMessage':
          return toJson(
            ExtensionResponseSchema,
            await handleDebugNativeMessage(
              context,
              extensionRequest.requestId,
              extensionRequest.request.value,
            ),
          )
        default: {
          log.warn('Unknown request type:', extensionRequest.request.case)
          return toJson(
            ExtensionResponseSchema,
            create(ExtensionResponseSchema, {
              requestId: extensionRequest.requestId,
              success: false,
              error: 'Unknown request type',
              response: { case: undefined },
            }),
          )
        }
      }
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      log.error('Error handling message:', errorMessage)
      return toJson(
        ExtensionResponseSchema,
        create(ExtensionResponseSchema, {
          requestId: (message as any)?.requestId || 'unknown',
          success: false,
          error: errorMessage,
          response: { case: undefined },
        }),
      )
    }
  }
}

export { createMessageDispatcher as createMessageHandler }
