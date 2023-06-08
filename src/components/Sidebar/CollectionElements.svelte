<script lang="ts">
  import CollectionElement from "@/components/Sidebar/CollectionElement.svelte";
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import type { Collection, CollectionElementsWithLabel } from "@/lib/types";
  import { writable } from "svelte/store";
  import { fade } from "svelte/transition";
  import SimpleBar from "simplebar";
  import Select from "@/components/UI/Select.svelte";
  import { collections } from "@/store/collections";
  import {
    commandAddElementsToCollection,
    commandRemoveElementsFromCollection,
  } from "@/lib/command";
  import RemoveElements from "@/components/Sidebar/RemoveElements.svelte";
  import { showInfoToast } from "@/lib/toast";
  import ADisclosure from "@/components/UI/ADisclosure.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  export let collection: Collection;
  export let collectionElement: CollectionElementsWithLabel[];

  $: selectedElements = collectionElement
    .flatMap((v) => v.elements)
    .filter((v) => checked[v.id]);

  let checked: Record<number, boolean> = {};
  let isCheckAll = writable(false);
  isCheckAll.subscribe(
    (val) =>
      (checked = collectionElement
        .flatMap((v) => v.elements)
        .reduce((acc, cur) => {
          acc[cur.id] = val;
          return acc;
        }, {} as Record<number, boolean>))
  );
  $: isCheckedAnyOne = Object.entries(checked).some(([id, val]) => val);

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
    checked = {};
  };

  const removeElements = async () => {
    await commandRemoveElementsFromCollection(
      collection.id,
      selectedElements.map((v) => v.id)
    );
    await sidebarCollectionElements.init(collection.id);
    showInfoToast("コレクションからの削除が完了しました");
    isOpenRemoveElements = false;
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
      {#if isCheckedAnyOne}
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
        {#if collectionElement.length === 1}
          {#each collectionElement[0].elements as ele (ele.id)}
            <CollectionElement
              checked={checked[ele.id]}
              on:check={(e) => {
                checked[ele.id] = e.detail.value;
                checked = checked;
              }}
              collectionElement={ele}
            />
          {/each}
        {:else}
          {#each collectionElement as { label, elements } (label)}
            <ADisclosure {label}>
              {#each elements as ele (ele.id)}
                <CollectionElement
                  checked={checked[ele.id]}
                  on:check={(e) => {
                    checked[ele.id] = e.detail.value;
                    checked = checked;
                  }}
                  collectionElement={ele}
                />
              {/each}
            </ADisclosure>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
<RemoveElements
  bind:isOpen={isOpenRemoveElements}
  {collection}
  removeElements={selectedElements}
  on:confirmRemove={removeElements}
/>
