import type { GetDmmPackIdsRequest as ExtGetPacksReq } from '@launcherg/shared/proto/extension_internal'
import type { HandlerContext } from '../shared/types'
import { create } from '@bufbuild/protobuf'
import { TimestampSchema } from '@bufbuild/protobuf/wkt'
import { ExtensionResponseSchema, GetDmmPackIdsResponseSchema as ExtGetPacksRes } from '@launcherg/shared/proto/extension_internal'
import { NativeMessageSchema, GetDmmPackIdsRequestSchema as NmGetPacksReq } from '@launcherg/shared/proto/native_messaging'

export async function handleGetDmmPackIds(
  context: HandlerContext,
  requestId: string,
  _req: ExtGetPacksReq,
) {
  const nativeMessage = create(NativeMessageSchema, {
    timestamp: create(TimestampSchema, { seconds: BigInt(Math.floor(Date.now() / 1000)) }),
    requestId: context.idGenerator.generate(),
    message: { case: 'getDmmPackIds', value: create(NmGetPacksReq, { extensionId: context.extensionId }) },
  })
  const nmRes = await context.nativeMessenger.send(nativeMessage)
  const storeIds = nmRes && nmRes.response?.case === 'dmmPackIds' ? nmRes.response.value.storeIds : []
  return create(ExtensionResponseSchema, {
    requestId,
    success: true,
    error: '',
    response: { case: 'getDmmPackIdsResult', value: create(ExtGetPacksRes, { storeIds }) },
  })
}
