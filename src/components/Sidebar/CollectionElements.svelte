<script lang="ts">
  import CollectionElement from "@/components/Sidebar/CollectionElement.svelte";
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import type {
    Collection,
    CollectionElement as TCollectionElement,
  } from "@/lib/types";
  import { writable } from "svelte/store";
  import { fade } from "svelte/transition";
  import SimpleBar from "simplebar";
  import Select from "@/components/UI/Select.svelte";
  import { collections } from "@/store/collections";
  import { commandAddElementsToCollection } from "@/lib/command";
  import RemoveElements from "@/components/Sidebar/RemoveElements.svelte";
  import { showInfoToast } from "@/lib/toast";

  export let collection: Collection;
  export let collectionElement: TCollectionElement[];

  $: selectedElements = collectionElement.filter((_, i) => checked[i]);

  let checked: boolean[] = [];
  let isCheckAll = writable(false);
  isCheckAll.subscribe((val) => (checked = collectionElement.map(() => val)));

  const simplebar = (node: HTMLElement) => {
    new SimpleBar(node, { scrollbarMinSize: 64 });
  };

  let copyDestinationCollectionId = 0;
  const copyElements = async () => {
    await commandAddElementsToCollection(
      copyDestinationCollectionId,
      selectedElements.map((v) => v.id)
    );
    showInfoToast(
      `${
        $collections.find((v) => v.id === copyDestinationCollectionId)?.name
      }へのゲーム追加が完了しました`
    );
    copyDestinationCollectionId = 0;
  };

  let isOpenRemoveElements = false;
</script>

<div class="grid-(~ rows-[min-content_1fr]) h-full overflow-y-hidden">
  {#if collectionElement.length}
    <div class="flex items-center">
      <!-- svelte-ignore a11y-label-has-associated-control -->
      <label
        class="w-12 h-8 flex-shrink-0 cursor-pointer flex items-center justify-center"
      >
        <Checkbox bind:value={$isCheckAll} />
      </label>
      {#if checked.some((v) => v)}
        <div
          transition:fade={{ duration: 150 }}
          class="flex items-center gap-2"
        >
          <Select
            title="Select copy destination"
            filterPlaceholder="Filter collections"
            enableFilter
            bind:value={copyDestinationCollectionId}
            options={$collections.map((v) => ({ label: v.name, value: v.id }))}
            showSelectedCheck={false}
            on:select={copyElements}
          >
            <Button
              variant="accent"
              text="Copy"
              tooltip={{
                content: "他のコレクションに選択したゲームを追加",
                theme: "default",
                placement: "bottom",
                delay: 1000,
              }}
            />
          </Select>
          <Button
            variant="error"
            text="Remove"
            tooltip={{
              content: "選択したゲームをこのコレクションから削除",
              theme: "default",
              placement: "bottom",
              delay: 1000,
            }}
            on:click={() => (isOpenRemoveElements = true)}
          />
        </div>
      {/if}
    </div>
    <div class="flex-1 mt-2 min-h-0">
      <div use:simplebar class="h-full overflow-y-auto">
        {#each collectionElement as ele, i (ele.id)}
          <CollectionElement
            checked={checked[i]}
            on:check={(e) => {
              checked[i] = e.detail.value;
              checked = checked;
            }}
            collectionElement={ele}
          />
        {/each}
      </div>
    </div>
  {/if}
</div>
<RemoveElements
  bind:isOpen={isOpenRemoveElements}
  {collection}
  removeElements={selectedElements}
/>
