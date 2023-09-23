<script lang="ts">
  import type { CollectionElement } from "@/lib/types";
  import { exists } from "@tauri-apps/api/fs";
  import { convertFileSrc } from "@tauri-apps/api/tauri";
  export let collectionElement: CollectionElement;

  $: existThumbnail = exists(collectionElement.thumbnail);
  $: imgSrc = convertFileSrc(collectionElement.thumbnail);
</script>

{#await existThumbnail then isExist}
  <div
    class="hover:scale-115 hover:shadow-md focus-within:scale-110 focus-within:shadow-md transition-all cursor-pointer"
  >
    <a
      tabIndex={0}
      href={`/works?id=${collectionElement.id}&gamename=${collectionElement.gamename}`}
    >
      {#if isExist}
        <img
          decoding="async"
          class="object-contain rounded"
          src={imgSrc}
          loading="lazy"
          alt={`${collectionElement.gamename}のサムネイル`}
        />
      {:else}
        <div
          class="text-(body text-primary) font-bold p-8 rounded border bg-bg-primary"
        >
          {collectionElement.gamename}
        </div>
      {/if}
    </a>
  </div>
{/await}
