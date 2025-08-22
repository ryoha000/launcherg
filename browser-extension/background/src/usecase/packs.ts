import type { GetDmmPackIdsRequest as ExtGetPacksReq } from '@launcherg/shared/proto/extension_internal'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { ExtensionResponseSchema, GetDmmPackIdsResponseSchema as ExtGetPacksRes } from '@launcherg/shared/proto/extension_internal'

export async function handleGetDmmPackIds(
  context: HandlerContext,
  requestId: string,
  _req: ExtGetPacksReq,
) {
  const nativeMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'GetDmmPackIds', value: { extension_id: context.extensionId } },
  }
  const nmRes = await (context.nativeMessenger as any).sendJson(nativeMessage)
  const storeIds = nmRes && nmRes.response?.case === 'DmmPackIds' ? nmRes.response.value.store_ids : []
  return create(ExtensionResponseSchema, {
    requestId,
    success: true,
    error: '',
    response: { case: 'getDmmPackIdsResult', value: create(ExtGetPacksRes, { storeIds }) },
  })
}
