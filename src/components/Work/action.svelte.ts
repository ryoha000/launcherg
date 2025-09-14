import type { WorkDetailsVm } from '@/lib/command'
import { get } from 'svelte/store'
import { commandLaunchWork, commandOpenUrl } from '@/lib/command'
import { useWorkLnkQuery } from '@/lib/data/queries/workLnk'
import { showErrorToast } from '@/lib/toast'
import { localStorageWritable } from '@/lib/utils'
import { startProcessMap } from '@/store/startProcessMap'

interface InstallOption {
  store: 'DMM' | 'DLsite'
  install: () => Promise<void>
}

export function useStart(workDetail: WorkDetailsVm) {
  const workLnkQuery = useWorkLnkQuery(workDetail.id)

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

  const isNotInstalled = $derived(get(workLnkQuery).data?.length === 0)

  const installFromDmm = async () => {
    const dmm = workDetail.dmm
    if (!dmm) {
      throw new Error('dmm is not set')
    }
    const payload = {
      type: 'download',
      value: {
        game: {
          storeId: dmm.storeId,
          category: dmm.category,
          subcategory: dmm.subcategory,
        },
      },
    }
    const url = new URL('https://dlsoft.dmm.co.jp/mylibrary/')
    url.searchParams.set('launcherg', JSON.stringify(payload))
    await commandOpenUrl(url.toString())
  }
  const installFromDlsite = async () => {
    const dlsite = workDetail.dlsite
    if (!dlsite) {
      throw new Error('dlsite is not set')
    }
    const payload = {
      type: 'download',
      value: {
        game: {
          storeId: dlsite.storeId,
          category: dlsite.category,
        },
      },
    }
    const url = new URL('https://play.dlsite.com/library')
    url.searchParams.set('launcherg', JSON.stringify(payload))
    await commandOpenUrl(url.toString())
  }
  const installOptions = $derived.by(() => {
    const options: InstallOption[] = []
    if (workDetail.dmm) {
      options.push({ store: 'DMM', install: installFromDmm })
    }
    if (workDetail.dlsite) {
      options.push({ store: 'DLsite', install: installFromDlsite })
    }
    return options
  })

  return { start, isNotInstalled, installOptions }
}
