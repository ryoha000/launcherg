<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import type { CollectionElement } from "@/lib/types";
  import { Link } from "svelte-routing";
  import { writable } from "svelte/store";
  import { fade } from "svelte/transition";

  export let collectionElement: CollectionElement[];

  let checked: boolean[] = [];
  let isCheckAll = writable(false);
  isCheckAll.subscribe((val) => (checked = collectionElement.map(() => val)));
</script>

<div>
  <div class="flex items-center">
    <!-- svelte-ignore a11y-label-has-associated-control -->
    <label
      class="w-12 h-8 flex-shrink-0 cursor-pointer flex items-center justify-center"
    >
      <Checkbox bind:value={$isCheckAll} />
    </label>
    {#if checked.some((v) => v)}
      <div transition:fade={{ duration: 150 }} class="flex items-center gap-2">
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
  {#each collectionElement as ele, i (ele.id)}
    <div
      class="flex items-center p-(r-4 y-1) rounded transition-all hover:bg-bg-secondary"
    >
      <!-- svelte-ignore a11y-label-has-associated-control -->
      <label
        class="w-12 h-12 flex-shrink-0 cursor-pointer flex items-center justify-center"
      >
        <Checkbox
          value={checked[i]}
          on:update={(e) => {
            checked[i] = e.detail.value;
            checked = checked;
          }}
        />
      </label>
      <Link
        to={`/works/${ele.id}`}
        class="flex-(~ 1) h-12 w-full items-center gap-4"
      >
        <img alt=" " src={ele.iconPath} class="h-10 w-10" />
        <div class="text-(body text-primary) font-bold">{ele.gamename}</div>
      </Link>
    </div>
  {/each}
</div>
