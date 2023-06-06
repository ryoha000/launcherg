<script lang="ts">
  import AddGameExplore from "@/components/Sidebar/AddGameExplore.svelte";
  import AddGameManual from "@/components/Sidebar/AddGameManual.svelte";
  import AddGamePopover from "@/components/Sidebar/AddGamePopover.svelte";
  import ChangeCollectionName from "@/components/Sidebar/ChangeCollectionName.svelte";
  import ChangePopover from "@/components/Sidebar/ChangePopover.svelte";
  import DeleteCollection from "@/components/Sidebar/DeleteCollection.svelte";
  import NewCollection from "@/components/Sidebar/NewCollection.svelte";
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
    if (collections.length) {
      if (collections.findIndex((v) => v.id === selectedCollectionId) === -1) {
        selectedCollectionId = collections[0].id;
      }
    }
  }

  $: {
    value = collections.find((v) => v.id === selectedCollectionId) ?? null;
  }

  $: collectionOptions = collections.map((v) => ({
    label: v.name,
    value: v.id,
  }));

  let isOpenNewCollection = false;
  let isOpenChangeCollectionName = false;
  let isOpenDeleteCollection = false;
  let isOpenAddGameExplore = false;
  let isOpenAddGameManual = false;
</script>

<div class="grid items-center gap-2 grid-cols-[1fr_min-content]">
  <Select
    title="Select display collection"
    filterPlaceholder="Filter collections"
    enableFilter
    bind:value={selectedCollectionId}
    options={collectionOptions}
    bottomCreateButtonText="Create new collection"
    on:create={() => (isOpenNewCollection = true)}
  />
  <div class="flex items-center relative">
    <APopover isRelativeRoot={false} panelClass="right-0" let:close>
      <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center rounded-r-0"
        tooltip={{
          content: "このコレクションにゲームを追加",
          placement: "bottom",
          theme: "default",
        }}
        slot="button"
      >
        <div class="color-ui-tertiary w-5 h-5 i-iconoir-plus" />
      </ButtonBase>
      <AddGamePopover
        on:close={() => close(null)}
        on:explore={() => {
          isOpenAddGameExplore = true;
          close(null);
        }}
        on:manual={() => {
          isOpenAddGameManual = true;
          close(null);
        }}
      />
    </APopover>
    <APopover isRelativeRoot={false} panelClass="right-0" let:close>
      <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center border-x-transparent rounded-0"
        tooltip={{
          content: "このコレクションの変更",
          placement: "bottom",
          theme: "default",
        }}
        slot="button"
      >
        <div
          class="color-ui-tertiary w-4 h-4 i-material-symbols-edit-rounded"
        />
      </ButtonBase>
      <ChangePopover
        showDelete={value?.id !== 1}
        on:close={() => close(null)}
        on:changeName={() => {
          isOpenChangeCollectionName = true;
          close(null);
        }}
        on:delete={() => {
          if (value?.id === 1) {
            return;
          }
          isOpenDeleteCollection = true;
          close(null);
        }}
      />
    </APopover>
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
<NewCollection bind:isOpen={isOpenNewCollection} />
<ChangeCollectionName
  bind:isOpen={isOpenChangeCollectionName}
  collection={value}
/>
<DeleteCollection bind:isOpen={isOpenDeleteCollection} collection={value} />
<AddGameExplore bind:isOpen={isOpenAddGameExplore} collection={value} />
<AddGameManual bind:isOpen={isOpenAddGameManual} collection={value} />
