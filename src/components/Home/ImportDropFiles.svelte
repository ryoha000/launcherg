<script lang='ts'>
  import type { WorkPathInput } from '@/lib/command'
  import type { AllGameCacheOne } from '@/lib/types'
  import { onDestroy, onMount } from 'svelte'
  import { useFileDrop } from '@/components/Home/fileDrop.svelte'
  import ImportManually from '@/components/Sidebar/ImportManually.svelte'
  import { commandRegisterWorkFromPath } from '@/lib/command'
  import { registerErogamescapeInformations } from '@/lib/registerErogamescapeInformations'
  import { showInfoToast } from '@/lib/toast'
  import { sidebarWorks } from '@/store/sidebarWorks'

  const { targetFileAccessor, popToTargetFile, startListening, stopListening } = useFileDrop()

  onMount(startListening)
  onDestroy(stopListening)

  let isOpenImportFileDrop = $derived(targetFileAccessor() !== undefined)

  const next = () => {
    popToTargetFile()
  }
  const importManually = async (
    exePath: string | null,
    lnkPath: string | null,
    gameCache: AllGameCacheOne,
  ) => {
    let path: WorkPathInput
    if (exePath) {
      path = { type: 'exe', exePath }
    }
    else {
      path = { type: 'lnk', lnkPath: lnkPath as string }
    }
    await commandRegisterWorkFromPath({ path, gameCache })
    await registerErogamescapeInformations()
    await sidebarWorks.refetch()
    showInfoToast(`${gameCache.gamename}を登録しました。`)
    isOpenImportFileDrop = false
    setTimeout(next, 0)
  }
  const skipImport = () => {
    isOpenImportFileDrop = false
    setTimeout(next, 0)
  }
</script>

<ImportManually
  bind:isOpen={isOpenImportFileDrop}
  path={targetFileAccessor()}
  cancelText='Skip'
  onconfirm={importManually}
  oncancel={skipImport}
/>
