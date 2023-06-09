<script lang="ts">
  import SearchAttribute from "@/components/Sidebar/SearchAttribute.svelte";
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import SortPopover from "@/components/Sidebar/SortPopover.svelte";
  import {
    type Attribute,
    type AttributeKey,
  } from "@/components/Sidebar/searchAttributes";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import APopover from "@/components/UI/APopover.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import ScrollableHorizontal from "@/components/UI/ScrollableHorizontal.svelte";
  import { createEventDispatcher } from "svelte";

  export let query: string;
  export let order: SortOrder;
  export let attributes: Attribute[];

  const dispatcher = createEventDispatcher<{
    toggleAttributeEnabled: { key: AttributeKey };
  }>();
</script>

<div class="space-y-1 w-full">
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
        <div
          class="color-ui-tertiary w-5 h-5 i-material-symbols-sort-rounded"
        />
      </ButtonBase>
      <SortPopover bind:value={order} on:close={() => close(null)} />
    </APopover>
  </div>
  <ScrollableHorizontal>
    <div class="flex items-center gap-2 pb-1">
      {#each attributes as attribute (attribute.key)}
        <SearchAttribute
          {attribute}
          on:click={() =>
            dispatcher("toggleAttributeEnabled", { key: attribute.key })}
        />
      {/each}
    </div>
  </ScrollableHorizontal>
</div>
