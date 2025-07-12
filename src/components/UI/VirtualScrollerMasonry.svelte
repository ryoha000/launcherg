<script lang='ts'>
  import type { Readable } from 'svelte/store'
  import type { CollectionElement } from '@/lib/types'
  import { onDestroy, onMount } from 'svelte'
  import ZappingGameItem from '@/components/Home/ZappingGameItem.svelte'
  import { useVirtualScrollerMasonry } from '@/components/UI/virtualScrollerMasonry'

  interface Props {
    elements: Readable<CollectionElement[]>
    setVirtualHeight: (v: number) => void
    contentsWidth: Readable<number>
    contentsScrollY: Readable<number>
    containerHeight: Readable<number>
    contentsScrollTo: (v: number) => void
  }

  const {
    elements,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight,
    contentsScrollTo,
  }: Props = $props()

  const { visibleLayouts } = useVirtualScrollerMasonry(
    elements,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight,
  )

  const LAST_CONTENTS_SCROLL_Y_KEY = 'lastContentsScrollY'
  onMount(() => {
    const lastContentsScrollY = localStorage.getItem(
      LAST_CONTENTS_SCROLL_Y_KEY,
    )
    if (lastContentsScrollY) {
      contentsScrollTo(+lastContentsScrollY)
    }
  })
  onDestroy(() => {
    localStorage.setItem(LAST_CONTENTS_SCROLL_Y_KEY, `${$contentsScrollY}`)
  })
</script>

<div>
  {#each $visibleLayouts as { top, left, width, height, element } (element.id)}
    <div
      class='absolute'
      style='left: {left}px; top: {top}px; width: {width}px; height: {height}px;'
    >
      <ZappingGameItem collectionElement={element} />
    </div>
  {/each}
</div>
