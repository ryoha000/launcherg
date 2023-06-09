<script lang="ts">
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import SortPopover from "@/components/Sidebar/SortPopover.svelte";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import APopover from "@/components/UI/APopover.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";

  export let query: string;
  export let order: SortOrder;
</script>

<div class="flex items-center gap-2">
  <div class="flex-1">
    <SearchInput
      bind:value={query}
      placeholder="Filter by title, brand and more"
    />
  </div>
  <APopover panelClass="right-0" let:close>
    <ButtonBase
      appendClass="h-8 w-8 flex items-center justify-center"
      tooltip={{
        content: "ゲームの並べ替え",
        placement: "bottom",
        theme: "default",
        delay: 1000,
      }}
      slot="button"
    >
      <div class="color-ui-tertiary w-5 h-5 i-material-symbols-sort-rounded" />
    </ButtonBase>
    <SortPopover bind:value={order} on:close={() => close(null)} />
  </APopover>
</div>
