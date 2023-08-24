<script lang="ts">
  import ImportAutomatically from "@/components/Sidebar/ImportAutomatically.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import ImportPopover from "@/components/Sidebar/ImportPopover.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import Button from "@/components/UI/Button.svelte";
  import { commandUpsertCollectionElement } from "@/lib/command";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { showInfoToast } from "@/lib/toast";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  let isOpenImportAutomatically = false;
  let isOpenImportManually = false;

  const importManually = async (arg: {
    id: number;
    gamename: string;
    path: string;
  }) => {
    await commandUpsertCollectionElement(arg.id, arg.gamename, arg.path);
    await registerCollectionElementDetails();
    await sidebarCollectionElements.refetch();
    isOpenImportManually = false;
    showInfoToast(`${arg.gamename}を登録しました。`);
  };
</script>

<div class="mt-4 w-full px-2 flex items-center">
  <div class="text-(text-primary body) font-bold pl-2 mr-auto">
    登録したゲーム
  </div>
  <APopover panelClass="right-0" let:close>
    <Button
      text="Add"
      leftIcon="i-material-symbols-computer-outline-rounded"
      appendClass="ml-auto"
      slot="button"
    />
    <ImportPopover
      on:close={() => close(null)}
      on:startAuto={() => (isOpenImportAutomatically = true)}
      on:startManual={() => (isOpenImportManually = true)}
    />
  </APopover>
</div>
{#if isOpenImportAutomatically}
  <ImportAutomatically bind:isOpen={isOpenImportAutomatically} />
{/if}
{#if isOpenImportManually}
  <ImportManually
    bind:isOpen={isOpenImportManually}
    on:confirm={(e) => importManually(e.detail)}
  />
{/if}
