<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props {
    collectionElement: CollectionElement
  }

  const { collectionElement }: Props = $props()

  const imgSrc = $derived(convertFileSrc(collectionElement.thumbnail))
</script>

<div
  class='relative h-full w-full cursor-pointer transition-all hover:z-10 focus-within:scale-110 hover:scale-115 focus-within:shadow-md hover:shadow-md'
>
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:route
    class='block h-full w-full'
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <img
        decoding='async'
        class='h-full w-full rounded object-contain'
        src={imgSrc}
        alt={`${collectionElement.gamename}のサムネイル`}
      />
    {:else}
      <div
        class='h-full w-full flex items-center justify-center border rounded bg-bg-primary px-6 text-(body text-primary) font-bold'
      >
        {collectionElement.gamename}
      </div>
    {/if}
  </a>
</div>
