import { commandCreateElementsInPc } from '@/lib/command'
import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
import { showInfoToast } from '@/lib/toast'
import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

export function useAutoImport() {
  let isLoading = $state(false)
  let useCache = $state(true)

  const executeImport = async (paths: string[]) => {
    isLoading = true
    try {
      const res = await commandCreateElementsInPc(paths, useCache)
      await registerCollectionElementDetails()
      await sidebarCollectionElements.refetch()

      const text = res.length
        ? `「${res[0]}」${
          res.length === 1 ? 'が' : `、他${res.length}件`
        }追加されました`
        : '新しく追加されたゲームはありません'

      showInfoToast(text)
      return true
    }
    finally {
      isLoading = false
    }
  }

  return {
    isLoading: () => isLoading,
    useCache: () => useCache,
    setUseCache: (value: boolean) => { useCache = value },
    executeImport,
  }
}
