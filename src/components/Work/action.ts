import type { WorkDetailsVm } from '@/lib/command'
import { get } from 'svelte/store'
import { commandPlayGame } from '@/lib/command'
import { showErrorToast } from '@/lib/toast'
import { localStorageWritable } from '@/lib/utils'
import { startProcessMap } from '@/store/startProcessMap'

export function useStart(workDetail: WorkDetailsVm) {
  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    'play-admin-cache',
    {},
  )

  const updateRunAsForCollectionElement = (collectionElementId: number, runAs: 'admin' | 'user') => {
    isAdminRecord.update((v) => {
      v[collectionElementId] = runAs === 'admin'
      return v
    })
  }
  const updateStartProcess = (id: number, processId: number) => {
    startProcessMap.update((v) => {
      v[id] = processId
      return v
    })
  }

  const start = async (runAs: 'admin' | 'user' | 'default') => {
    const collectionElementId = workDetail.collectionElementId
    if (!collectionElementId) {
      throw new Error('collectionElementId is not set')
    }
    // runAs に指定があれば、次からの runAs === 'default' ではその権限で実行する
    if (runAs === 'admin' || runAs === 'user') {
      if (collectionElementId) {
        updateRunAsForCollectionElement(collectionElementId, runAs)
      }
    }

    // 過去に実行された権限の記録もなく、runAs も default の場合は user で実行する
    let isAdmin = false
    switch (runAs) {
      case 'admin':
        isAdmin = true
        break
      case 'user':
        isAdmin = false
        break
      case 'default': {
        if (collectionElementId) {
          const cache = get(isAdminRecord)[collectionElementId]
          if (cache) {
            isAdmin = cache
          }
        }
        break
      }
      default:
        throw new Error(`Invalid runAs: ${runAs satisfies never}`)
    }

    try {
      // TODO: これからは実行可能なものが複数存在するケースも発生しうるため、複数ある場合はダイアログから選ばせる。引数でそのパスだかそのパスのIDだかを受け取る
      const processId = await commandPlayGame(collectionElementId, isAdmin)
      if (processId) {
        updateStartProcess(collectionElementId, processId)
      }
    }
    catch (e) {
      showErrorToast(e as string)
    }
  }

  return { start }
}
