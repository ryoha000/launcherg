<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { link } from "svelte-spa-router";

  export let collectionElement: CollectionElement;

  $: iconSrc = convertFileSrc(collectionElement.icon);
</script>

<div
  class="flex items-center py-1 pl-2 rounded transition-all hover:bg-bg-secondary overflow-hidden"
>
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class="flex-(~ 1) h-12 w-full items-center gap-2 pr-2"
    use:link
  >
    <img
      alt="{collectionElement.gamename}_icon"
      src="{iconSrc}?{collectionElement.registeredAt}"
      class="h-10 w-10 rounded"
      loading="lazy"
    />
    <div class="text-(body text-primary) font-bold max-h-full">
      {collectionElement.gamename}
    </div>
  </a>
</div>
