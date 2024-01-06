<script lang="ts">
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import { useVirtualScrollerMasonry } from "@/components/UI/virtualScrollerMasonry";
  import type { CollectionElement } from "@/lib/types";
  import type { Readable } from "svelte/store";

  export let elements: Readable<CollectionElement[]>;
  export let setVirtualHeight: (v: number) => void;
  export let contentsWidth: Readable<number>;
  export let contentsScrollY: Readable<number>;
  export let containerHeight: Readable<number>;

  const { visibleLayouts } = useVirtualScrollerMasonry(
    elements,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight
  );
</script>

<div>
  {#each $visibleLayouts as { top, left, width, height, element } (element.id)}
    <div
      class="absolute"
      style="left: {left}px; top: {top}px; width: {width}px; height: {height}px;"
    >
      <ZappingGameItem collectionElement={element} />
    </div>
  {/each}
</div>
