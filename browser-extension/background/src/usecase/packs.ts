import type { ExtensionResponse, GetDmmOmitWorksRequest as ExtGetOmitReq } from '@launcherg/shared'
import type { DmmOmitWorkItemTs, NativeMessageTs } from '@launcherg/shared/typeshare/native-messaging'
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
  const nmRes = await (context.nativeMessenger as any).sendJson(nativeMessage)
  const storeIds = nmRes && nmRes.response?.case === 'DmmOmitWorks'
    ? (nmRes.response.value as DmmOmitWorkItemTs[]).map(i => i.dmm.store_id)
    : []
  return {
    requestId,
    success: true,
    error: '',
    response: { case: 'getDmmOmitWorksResult', value: { storeIds } },
  }
}
