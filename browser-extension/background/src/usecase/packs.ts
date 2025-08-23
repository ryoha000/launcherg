import type { ExtensionResponse, GetDmmPackIdsRequest as ExtGetPacksReq } from '@launcherg/shared'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'

export async function handleGetDmmPackIds(
  context: HandlerContext,
  requestId: string,
  _req: ExtGetPacksReq,
): Promise<ExtensionResponse> {
  const nativeMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'GetDmmPackIds', value: { extension_id: context.extensionId } },
  }
  const nmRes = await (context.nativeMessenger as any).sendJson(nativeMessage)
  const storeIds = nmRes && nmRes.response?.case === 'DmmPackIds' ? nmRes.response.value.store_ids : []
  return {
    requestId,
    success: true,
    error: '',
    response: { case: 'getDmmPackIdsResult', value: { storeIds } },
  }
}
