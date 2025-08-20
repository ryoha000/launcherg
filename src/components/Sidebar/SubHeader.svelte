<script lang='ts'>
  import type { AllGameCacheOne } from '@/lib/types'
  import ImportAutomatically from '@/components/Sidebar/ImportAutomatically.svelte'
  import ImportManually from '@/components/Sidebar/ImportManually.svelte'
  import ImportPopover from '@/components/Sidebar/ImportPopover.svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import { commandUpsertCollectionElement } from '@/lib/command'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { showInfoToast } from '@/lib/toast'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  let isOpenImportAutomatically = $state(false)
  let isOpenImportManually = $state(false)

  const importManually = async (
    exePath: string | null,
    lnkPath: string | null,
    gameCache: AllGameCacheOne,
  ) => {
    await commandUpsertCollectionElement({ exePath, lnkPath, gameCache })
    await registerCollectionElementDetails()
    await sidebarCollectionElements.refetch()
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
