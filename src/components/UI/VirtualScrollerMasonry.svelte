<script lang="ts">
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import { useVirtualScrollerMasonry } from "@/components/UI/virtualScrollerMasonry";
  import type { CollectionElement } from "@/lib/types";
  import { onDestroy, onMount } from "svelte";
  import type { Readable } from "svelte/store";

  export let elements: Readable<CollectionElement[]>;
  export let setVirtualHeight: (v: number) => void;
  export let contentsWidth: Readable<number>;
  export let contentsScrollY: Readable<number>;
  export let containerHeight: Readable<number>;
  export let contentsScrollTo: (v: number) => void;

  const { visibleLayouts } = useVirtualScrollerMasonry(
    elements,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight
  );

  const LAST_CONTENTS_SCROLL_Y_KEY = "lastContentsScrollY";
  onMount(() => {
    const lastContentsScrollY = localStorage.getItem(
      LAST_CONTENTS_SCROLL_Y_KEY
    );
    if (lastContentsScrollY) {
      contentsScrollTo(+lastContentsScrollY);
    }
  });
  onDestroy(() => {
    localStorage.setItem(LAST_CONTENTS_SCROLL_Y_KEY, `${$contentsScrollY}`);
  });
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
