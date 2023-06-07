<script lang="ts">
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/tauri";
  import { createEventDispatcher } from "svelte";
  import { link } from "svelte-spa-router";

  export let collectionElement: CollectionElement;
  export let checked: boolean;

  const dispather = createEventDispatcher<{ check: { value: boolean } }>();

  $: iconSrc = convertFileSrc(collectionElement.icon);
</script>

<div
  class="flex items-center py-1 rounded transition-all hover:bg-bg-secondary overflow-hidden"
>
  <!-- svelte-ignore a11y-label-has-associated-control -->
  <label
    class="w-12 h-12 flex-shrink-0 cursor-pointer flex items-center justify-center"
  >
    <Checkbox
      value={checked}
      on:update={(e) => dispather("check", { value: e.detail.value })}
    />
  </label>
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class="flex-(~ 1) h-12 w-full items-center gap-2 pr-2"
    use:link
  >
    <img
      alt="{collectionElement.gamename}_icon"
      src={iconSrc}
      class="h-10 w-10"
    />
    <div class="text-(body text-primary) font-bold max-h-full">
      {collectionElement.gamename}
    </div>
  </a>
</div>
