<script lang="ts">
  import SortPopover from "@/components/Sidebar/SortPopover.svelte";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import APopover from "@/components/UI/APopover.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import Select from "@/components/UI/Select.svelte";
  import type { Collection } from "@/lib/types";

  export let collections: Collection[] = [];
  export let order: SortOrder;

  let selectedCollectionId = 0;
  export let value: Collection | null = null;

  $: {
    if (!selectedCollectionId && collections.length) {
      selectedCollectionId = collections[0].id;
    }
  }

  $: {
    value = collections.find((v) => v.id === selectedCollectionId) ?? null;
  }

  $: collectionOptions = collections.map((v) => ({
    label: v.name,
    value: v.id,
  }));
</script>

<div class="grid items-center gap-2 grid-cols-[1fr_min-content]">
  <Select
    title="Select display collection"
    filterPlaceholder="Filter collections"
    enableFilter
    bind:value={selectedCollectionId}
    options={collectionOptions}
  />
  <div class="flex items-center relative">
    <ButtonBase
      appendClass="h-8 w-8 flex items-center justify-center rounded-r-0"
      tooltip={{
        content: "このコレクションにゲームを追加",
        placement: "bottom",
        theme: "default",
        trigger: "click",
      }}
    >
      <div class="color-ui-tertiary w-5 h-5 i-iconoir-plus" />
    </ButtonBase>
    <ButtonBase
      appendClass="h-8 w-8 flex items-center justify-center border-x-transparent rounded-0"
      tooltip={{
        content: "このコレクションの名前を編集",
        placement: "bottom",
        theme: "default",
      }}
      on:click
    >
      <div class="color-ui-tertiary w-4 h-4 i-material-symbols-edit-rounded" />
    </ButtonBase>
    <APopover isRelativeRoot={false} panelClass="right-0" let:close>
      <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center rounded-l-0"
        tooltip={{
          content: "ゲームの並べ替え",
          placement: "bottom",
          theme: "default",
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
</div>
