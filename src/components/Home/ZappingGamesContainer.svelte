<script lang="ts">
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import { useMasonry } from "@/components/UI/masonry";
  import type { CollectionElement } from "@/lib/types";

  export let elements: CollectionElement[];
  const {
    masonry,
    masonryHeader,
    masonryContents,
    virtualHeight,
    visibleLayouts,
  } = useMasonry(elements);
</script>

<div use:masonry class="w-full h-full overflow-y-auto relative">
  <div use:masonryHeader>
    <slot name="header" />
  </div>
  <div class="p-(8 t-0) space-y-2">
    <h3 use:masonryHeader class="text-(text-primary h3) font-medium">
      登録したゲーム
    </h3>
    <div
      use:masonryContents
      class="relative transform-gpu backface-hidden"
      style="height: {$virtualHeight}px;"
    >
      {#each $visibleLayouts as { top, left, width, height, element } (element.id)}
        <div
          class="absolute"
          style="transform: translate({left}px, {top}px); width: {width}px; height: {height}px;"
        >
          <ZappingGameItem collectionElement={element} />
        </div>
      {/each}
    </div>
  </div>
</div>

<!-- <div
  use:masonry
  class="space-y-2 overflow-y-scroll relative"
  style="height: {$virtualHeight}px;"
>
  <h3 use:masonryHeader class="text-(text-primary h3) font-medium">
    登録したゲーム
  </h3> -->
<!-- <div class="relative">
    {#each $visibleLayouts as layout, column (column)}
      {#each layout as { top, width, height, element } (element.id)}
        <div
          class="absolute"
          style="top: {top}px; left: {column *
            width}px; width: {width}px; height: {height}px;"
        >
          <ZappingGameItem collectionElement={element} />
        </div>
      {/each}
    {/each}
  </div>
</div> -->
