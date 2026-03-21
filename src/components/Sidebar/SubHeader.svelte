<script lang='ts'>
  import type { WorkPathInput } from '@/lib/command'
  import type { AllGameCacheOne } from '@/lib/types'
  import ImportAutomatically from '@/components/Sidebar/ImportAutomatically.svelte'
  import ImportManually from '@/components/Sidebar/ImportManually.svelte'
  import ImportPopover from '@/components/Sidebar/ImportPopover.svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import { commandRegisterWorkFromPath } from '@/lib/command'
  import { registerErogamescapeInformations } from '@/lib/registerErogamescapeInformations'
  import { showInfoToast } from '@/lib/toast'
  import { sidebarWorks } from '@/store/sidebarWorks'

  let isOpenImportAutomatically = $state(false)
  let isOpenImportManually = $state(false)

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
    isOpenImportManually = false
    showInfoToast(`${gameCache.gamename}を登録しました。`)
  }
</script>

<div class='mt-4 w-full flex items-center px-2'>
  <div class='mr-auto pl-2 text-(body text-primary) font-bold'>
    登録したゲーム
  </div>
  <div class='flex items-center gap-2'>
    <!-- DLStore機能は廃止 -->
    <APopover panelClass='right-0'>
      {#snippet button()}
        <Button
          text='Add'
          leftIcon='i-material-symbols-computer-outline-rounded'
          appendClass='ml-auto'
        />
      {/snippet}
      {#snippet children({ close })}
        <ImportPopover
          onclose={() => close(null)}
          onstartAuto={() => (isOpenImportAutomatically = true)}
          onstartManual={() => (isOpenImportManually = true)}
        />
      {/snippet}
    </APopover>
  </div>
</div>
<ImportAutomatically bind:isOpen={isOpenImportAutomatically} />
<ImportManually
  bind:isOpen={isOpenImportManually}
  onconfirm={importManually}
  oncancel={() => (isOpenImportManually = false)}
/>
<!-- DLStoreManager 削除 -->
