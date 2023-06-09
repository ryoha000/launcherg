<script lang="ts">
  import ImportAutomatically from "@/components/Sidebar/ImportAutomatically.svelte";
  import ImportPopover from "@/components/Sidebar/ImportPopover.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import Button from "@/components/UI/Button.svelte";

  let isOpenImportAutomatically = false;
  let isOpenImportManually = false;
</script>

<div class="mt-4 w-full px-2 flex items-center">
  <div class="text-(text-primary body) font-bold pl-2 mr-auto">
    登録したゲーム
  </div>
  <APopover panelClass="right-0" let:close>
    <Button
      text="Add"
      leftIcon="i-material-symbols-computer-outline-rounded"
      variant="success"
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
<ImportAutomatically bind:isOpen={isOpenImportAutomatically} />
