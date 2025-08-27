import type { ExtensionResponse, GetDmmOmitWorksRequest as ExtGetOmitReq } from '@launcherg/shared'
import type { NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
import type { HandlerContext } from '../shared/types'

export async function handleGetDmmOmitWorks(
  context: HandlerContext,
  requestId: string,
  _req: ExtGetOmitReq,
): Promise<ExtensionResponse> {
  const nativeMessage: NativeMessageTs = {
    request_id: context.idGenerator.generate(),
    message: { case: 'GetDmmOmitWorks', value: { extension_id: context.extensionId } },
  }
  const nmRes = await context.nativeMessenger.sendJson(nativeMessage)
  const items = (nmRes && nmRes.response?.case === 'DmmOmitWorks'
    ? (nmRes.response.value)
    : [])
    .map(i => ({ workId: i.work_id, dmm: { storeId: i.dmm.store_id, category: i.dmm.category, subcategory: i.dmm.subcategory } }))
  return {
    requestId,
    success: true,
    error: '',
    response: { case: 'getDmmOmitWorksResult', value: { items } },
  }
}
