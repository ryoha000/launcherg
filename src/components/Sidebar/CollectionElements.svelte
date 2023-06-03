<script lang="ts">
  import CollectionElement from "@/components/Sidebar/CollectionElement.svelte";
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import type { CollectionElement as TCollectionElement } from "@/lib/types";
  import { writable } from "svelte/store";
  import { fade } from "svelte/transition";

  export let collectionElement: TCollectionElement[];

  let checked: boolean[] = [];
  let isCheckAll = writable(false);
  isCheckAll.subscribe((val) => (checked = collectionElement.map(() => val)));
</script>

<div class="grid-(~ rows-[min-content_1fr]) h-full">
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
          <Button
            variant="accent"
            text="Forward"
            tooltip={{
              content: "他のコレクションに選択したゲームを追加",
              theme: "default",
              placement: "bottom",
            }}
          />
          <Button
            variant="error"
            text="Remove"
            tooltip={{
              content: "選択したゲームをこのコレクションから削除",
              theme: "default",
              placement: "bottom",
            }}
          />
        </div>
      {/if}
    </div>
    <div class="flex-1 mt-2 scrollbar-gutter overflow-y-auto">
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
  {/if}
</div>
