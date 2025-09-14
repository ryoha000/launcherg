import type { CreateQueryResult } from '@tanstack/svelte-query'
import type { WorkDetailsVm } from '@/lib/command'
import { get } from 'svelte/store'
import { commandLaunchWork } from '@/lib/command'
import { showErrorToast } from '@/lib/toast'
import { localStorageWritable } from '@/lib/utils'
import { startProcessMap } from '@/store/startProcessMap'

export function useStart(workDetail: WorkDetailsVm, workLnkQuery: CreateQueryResult<[number, string][], Error>) {
  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    'play-admin-cache',
    {},
  )

  const updateRunAsForWork = (workId: number, runAs: 'admin' | 'user') => {
    isAdminRecord.update((v) => {
      v[workId] = runAs === 'admin'
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
    const workId = workDetail.id
    if (!workId) {
      throw new Error('workId is not set')
    }
    // runAs に指定があれば、次からの runAs === 'default' ではその権限で実行する
    if (runAs === 'admin' || runAs === 'user') {
      updateRunAsForWork(workId, runAs)
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
        if (workId) {
          const cache = get(isAdminRecord)[workId]
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
      // TODO: これからは実行可能なものが複数存在するケースも発生しうるため、複数ある場合はダイアログから選ばせる。現状は最初の lnk を取得して起動
      const list = get(workLnkQuery).data
      if (!list || list.length === 0) {
        throw new Error('起動可能なショートカットが登録されていません')
      }
      const [lnkId] = list[0]
      const processId = await commandLaunchWork(isAdmin, lnkId)
      if (processId) {
        updateStartProcess(workId, processId)
      }
    }
    catch (e) {
      showErrorToast(e as string)
    }
  }

  return { start }
}
